#!/bin/bash
# One-time import of projects from CLI registry to unified database

DB_PATH="data/loglens.db"
REGISTRY_PATH="$HOME/.config/loglens/projects.json"

if [ ! -f "$REGISTRY_PATH" ]; then
    echo "No CLI registry found at $REGISTRY_PATH"
    exit 0
fi

echo "🔄 Importing projects from CLI registry to database..."
echo "   Registry: $REGISTRY_PATH"
echo "   Database: $DB_PATH"
echo ""

# Read JSON and insert projects
jq -r '.projects | to_entries[] | @json' "$REGISTRY_PATH" | while read -r project; do
    id=$(echo "$project" | jq -r '.key')
    name=$(echo "$project" | jq -r '.value.name')
    root_path=$(echo "$project" | jq -r '.value.root_path')
    loglens_config=$(echo "$project" | jq -r '.value.loglens_config')
    last_accessed=$(echo "$project" | jq -r '.value.last_accessed')

    # Check if project already exists
    exists=$(sqlite3 "$DB_PATH" "SELECT COUNT(*) FROM projects WHERE id='$id';")

    if [ "$exists" -eq 0 ]; then
        # Insert new project
        sqlite3 "$DB_PATH" "INSERT INTO projects (id, name, root_path, loglens_config, last_accessed, description) VALUES ('$id', '$name', '$root_path', '$loglens_config', '$last_accessed', 'Imported from CLI registry');"
        echo "✓ Imported: $name ($root_path)"
    else
        # Update existing project
        sqlite3 "$DB_PATH" "UPDATE projects SET root_path='$root_path', loglens_config='$loglens_config', last_accessed='$last_accessed' WHERE id='$id';"
        echo "✓ Updated: $name ($root_path)"
    fi
done

echo ""
echo "✅ Import complete!"
total=$(sqlite3 "$DB_PATH" "SELECT COUNT(*) FROM projects;")
echo "📊 Total projects in database: $total"
