#!/bin/bash
set -e

echo "Waiting for PostgreSQL to be ready..."
# Use PGPASSWORD to avoid interactive prompt
export PGPASSWORD='password'
for i in {1..30}; do
    if psql -h localhost -U postgres -d backend_development -c "SELECT 1" > /dev/null 2>&1; then
        echo "PostgreSQL is ready!"
        break
    fi
    echo "Waiting for PostgreSQL to start... ($i/30)"
    sleep 1
done

# Create loco user for the application if it doesn't exist
psql -h localhost -U postgres -d backend_development -c "DO \$\$ BEGIN IF NOT EXISTS (SELECT FROM pg_catalog.pg_user WHERE usename = 'loco') THEN CREATE USER loco WITH PASSWORD 'loco' CREATEDB; END IF; END \$\$;"

# Create interlock database if it doesn't exist
psql -h localhost -U postgres -d backend_development -tc "SELECT 1 FROM pg_database WHERE datname = 'interlock'" | grep -q 1 || \
    psql -h localhost -U postgres -d backend_development -c "CREATE DATABASE interlock OWNER loco;"

# Create test database if it doesn't exist
psql -h localhost -U postgres -d backend_development -tc "SELECT 1 FROM pg_database WHERE datname = 'backend_test'" | grep -q 1 || \
    psql -h localhost -U postgres -d backend_development -c "CREATE DATABASE backend_test OWNER loco;"

echo "PostgreSQL setup complete!"

# Fix permissions
echo "Fixing permissions..."
sudo chown -R vscode:vscode /workspaces

echo "Setup complete!"
