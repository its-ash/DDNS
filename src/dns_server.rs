use hickory_proto::rr::{Name, RData, Record, RecordType};
use hickory_proto::op::{Header, OpCode, ResponseCode};
use hickory_server::server::{Request, RequestHandler, ResponseHandler, ResponseInfo};
use hickory_server::authority::MessageResponseBuilder;
use sqlx::SqlitePool;
use std::net::{Ipv4Addr, Ipv6Addr};
use std::str::FromStr;
use log::{info, warn, debug};

pub struct DdnsAuthority {
    pool: SqlitePool,
    origin: Name,
    ns_server: Name,
    admin_email: Name,
}

impl DdnsAuthority {
    pub fn new(pool: SqlitePool, domain: &str) -> Self {
        let origin = Name::from_str(&format!("{}.", domain)).unwrap();
        let ns_server = Name::from_str(&format!("ns.{}.", domain)).unwrap();
        let admin_email = Name::from_str(&format!("admin.{}.", domain)).unwrap();
        
        Self {
            pool,
            origin,
            ns_server,
            admin_email,
        }
    }

    async fn lookup_host(&self, name: &Name) -> Option<String> {
        let hostname = name.to_string().trim_end_matches('.').to_string();
        
        debug!("DNS lookup for: {}", hostname);
        
        let result = sqlx::query!(
            "SELECT current_ip FROM hosts WHERE hostname = ?",
            hostname
        )
        .fetch_optional(&self.pool)
        .await;

        match result {
            Ok(Some(record)) => {
                if let Some(ip) = record.current_ip {
                    info!("Found IP {} for {}", ip, hostname);
                    Some(ip)
                } else {
                    warn!("No IP address set for {}", hostname);
                    None
                }
            }
            Ok(None) => {
                debug!("Hostname {} not found in database", hostname);
                None
            }
            Err(e) => {
                warn!("Database error looking up {}: {}", hostname, e);
                None
            }
        }
    }

    fn create_soa_record(&self) -> Record {
        let mut record = Record::new();
        record.set_name(self.origin.clone());
        record.set_rr_type(RecordType::SOA);
        record.set_dns_class(hickory_proto::rr::DNSClass::IN);
        record.set_ttl(3600);
        
        let soa = hickory_proto::rr::rdata::SOA::new(
            self.ns_server.clone(),
            self.admin_email.clone(),
            1,      // serial
            3600,   // refresh
            600,    // retry
            86400,  // expire
            60,     // minimum
        );
        
        record.set_data(Some(RData::SOA(soa)));
        record
    }

    fn create_ns_record(&self) -> Record {
        let mut record = Record::new();
        record.set_name(self.origin.clone());
        record.set_rr_type(RecordType::NS);
        record.set_dns_class(hickory_proto::rr::DNSClass::IN);
        record.set_ttl(3600);
        record.set_data(Some(RData::NS(hickory_proto::rr::rdata::NS(self.ns_server.clone()))));
        record
    }
}

#[async_trait::async_trait]
impl RequestHandler for DdnsAuthority {
    async fn handle_request<R: ResponseHandler>(
        &self,
        request: &Request,
        mut response_handle: R,
    ) -> ResponseInfo {
        let builder = MessageResponseBuilder::from_message_request(request);
        let mut header = Header::response_from_request(request.header());
        
        header.set_authoritative(true);
        
        if request.op_code() != OpCode::Query {
            header.set_response_code(ResponseCode::NotImp);
            let response = builder.build_no_records(header);
            return response_handle.send_response(response).await.unwrap();
        }

        let query = request.query();
        let qname = query.name();
        let qtype = query.query_type();

        debug!("DNS query: {} {:?}", qname, qtype);

        // Handle SOA query
        if qtype == RecordType::SOA {
            let soa_record = self.create_soa_record();
            let answers = [soa_record];
            let response = builder.build(header, &answers, &[], &[], &[]);
            return response_handle.send_response(response).await.unwrap();
        }

        // Handle NS query
        if qtype == RecordType::NS {
            let ns_record = self.create_ns_record();
            let answers = [ns_record];
            let response = builder.build(header, &answers, &[], &[], &[]);
            return response_handle.send_response(response).await.unwrap();
        }

        // Handle A and AAAA queries
        if qtype == RecordType::A || qtype == RecordType::AAAA || qtype == RecordType::ANY {
            if let Some(ip_str) = self.lookup_host(&qname.into()).await {
                let mut records = Vec::new();

                // Try to parse as IPv6
                if let Ok(ipv6) = Ipv6Addr::from_str(&ip_str) {
                    if qtype == RecordType::AAAA || qtype == RecordType::ANY {
                        let mut record = Record::new();
                        record.set_name(Name::from(qname.clone()));
                        record.set_rr_type(RecordType::AAAA);
                        record.set_dns_class(hickory_proto::rr::DNSClass::IN);
                        record.set_ttl(60); // Short TTL for dynamic DNS
                        record.set_data(Some(RData::AAAA(hickory_proto::rr::rdata::AAAA(ipv6))));
                        records.push(record);
                    }
                }
                // Try to parse as IPv4
                else if let Ok(ipv4) = Ipv4Addr::from_str(&ip_str) {
                    if qtype == RecordType::A || qtype == RecordType::ANY {
                        let mut record = Record::new();
                        record.set_name(Name::from(qname.clone()));
                        record.set_rr_type(RecordType::A);
                        record.set_dns_class(hickory_proto::rr::DNSClass::IN);
                        record.set_ttl(60); // Short TTL for dynamic DNS
                        record.set_data(Some(RData::A(hickory_proto::rr::rdata::A(ipv4))));
                        records.push(record);
                    }
                }

                if !records.is_empty() {
                    let response = builder.build(header, records.iter(), &[], &[], &[]);
                    return response_handle.send_response(response).await.unwrap();
                }
            }
        }

        // No records found - return NXDOMAIN
        header.set_response_code(ResponseCode::NXDomain);
        let soa_record = self.create_soa_record();
        let additionals = [soa_record];
        let response = builder.build(header, &[], &[], &[], &additionals);
        response_handle.send_response(response).await.unwrap()
    }
}

pub async fn start_dns_server(pool: SqlitePool, domain: String) -> Result<(), Box<dyn std::error::Error>> {
    let dns_port = std::env::var("DNS_PORT").unwrap_or_else(|_| "5353".to_string());
    let bind_addr = format!("0.0.0.0:{}", dns_port);
    
    info!("Starting DNS server on {}", bind_addr);
    info!("Authoritative for zone: {}", domain);
    
    let handler = DdnsAuthority::new(pool, &domain);
    
    let mut server = hickory_server::ServerFuture::new(handler);
    
    let socket_addr: std::net::SocketAddr = bind_addr.parse()?;
    let udp_socket = tokio::net::UdpSocket::bind(socket_addr).await?;
    
    server.register_socket(udp_socket);
    
    info!("DNS server listening on UDP {}", bind_addr);
    
    server.block_until_done().await?;
    
    Ok(())
}
