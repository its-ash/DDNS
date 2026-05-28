# Multi-Hostname Feature

## Overview

The DDNS server now supports multiple hostnames sharing the same configuration (credentials and IP address). This is useful when you want multiple domain names to point to the same IP address without having to manage separate credentials for each hostname.

## How It Works

### Architecture

The system now uses two tables:

1. **configs** - Stores credentials (username/password) and the current IP address
2. **hosts** - Stores hostnames that reference a config

Each config can have multiple hostnames associated with it. When you update the IP using any set of credentials, all hostnames under that config will resolve to the new IP.

### Usage

#### Creating a New Host with New Credentials

When you create a new host from the dashboard without specifying an existing config, the system will:
1. Generate new credentials (username/password)
2. Create a new config with those credentials
3. Create the hostname linked to that config

#### Adding Additional Hostnames to Existing Config

On the dashboard, each config card shows:
- The config ID and credentials
- All hostnames using that config
- A form to add additional hostnames

To add a hostname to an existing config:
1. Locate the config card you want to add to
2. Enter a subdomain in the "Add another hostname" form
3. Click "ADD HOSTNAME"

The new hostname will immediately resolve to the same IP as other hostnames in that config.

### Example Use Case

You have a home server and want multiple domains pointing to it:

1. Create first hostname: `home.example.com`
   - System generates credentials: `abc123xyz456:SecretPass1234`
   
2. Add additional hostnames to the same config:
   - `nas.example.com`
   - `media.example.com`
   - `cloud.example.com`

All four hostnames share the same credentials and resolve to the same IP. Update your IP once using `abc123xyz456:SecretPass1234` and all four hostnames are updated.

### API Compatibility

The `/nic/update` endpoint remains fully compatible with existing routers and clients:

```bash
curl -u USERNAME:PASSWORD "http://yourdomain.com/nic/update?myip=1.2.3.4"
```

This updates the IP for **all hostnames** associated with those credentials.

### Migration from Old Schema

If you have an existing database with the old schema (one hostname per credential), use the migration script:

```bash
sqlite3 ddns.db < migrate_db.sql
```

This will:
1. Create the new configs table
2. Create the new hosts table
3. Migrate your existing data (one config per existing host)
4. Preserve all existing hostnames and credentials

After migration, you can add additional hostnames to any config.

### DNS Behavior

When a DNS query is made for a hostname:
1. System looks up the hostname in the `hosts` table
2. Joins with `configs` table to get the current IP
3. Returns A or AAAA record with the IP

All hostnames in the same config resolve to the same IP.

### Dashboard Features

The new dashboard displays:
- Configs grouped together with their credentials
- All hostnames under each config
- Current IP and last update time per config
- Ability to add new hostnames to existing configs
- Individual hostname deletion (config is deleted when last hostname is removed)

## Benefits

1. **Easier Management** - One set of credentials for multiple hostnames
2. **Single Update** - Update IP once, all hostnames update
3. **Flexible** - Add/remove hostnames without changing credentials
4. **Backward Compatible** - Existing setups continue to work
5. **Cost Effective** - No need for multiple DDNS accounts

## Technical Details

### Database Schema

**configs table:**
```sql
CREATE TABLE configs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username TEXT NOT NULL UNIQUE,
    password TEXT NOT NULL,
    current_ip TEXT,
    last_update DATETIME,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);
```

**hosts table:**
```sql
CREATE TABLE hosts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    hostname TEXT NOT NULL UNIQUE,
    config_id INTEGER NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (config_id) REFERENCES configs(id) ON DELETE CASCADE
);
```

### Cascade Deletion

When a config is deleted, all associated hostnames are automatically removed (ON DELETE CASCADE).

When individual hostnames are deleted, the config remains unless it was the last hostname.
