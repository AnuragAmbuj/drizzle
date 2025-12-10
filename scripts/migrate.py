import os
import sys
import psycopg2

def run_migration():
    db_url = os.environ.get("DATABASE_URL")
    if not db_url:
        print("DATABASE_URL not set")
        sys.exit(1)
    
    migration_file = "control-plane/crates/storage/migrations/20251210000000_create_security_config.sql"
    
    try:
        conn = psycopg2.connect(db_url)
        cur = conn.cursor()
        
        with open(migration_file, 'r') as f:
            sql = f.read()
            print(f"Executing: {sql}")
            cur.execute(sql)
            
        conn.commit()
        cur.close()
        conn.close()
        print("Migration applied successfully.")
    except Exception as e:
        print(f"Error: {e}")
        sys.exit(1)

if __name__ == "__main__":
    run_migration()
