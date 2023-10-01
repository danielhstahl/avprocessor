cargo run 
curl -X PUT http://127.0.0.1:8000/config -H 'Content-Type: application/json' -d @example_json.json

curl -X GET http://127.0.0.1:8000/config/latest

../../third_party/camilla/camilladsp -v -p1234 test.yaml




#plotcamillaconf test.yml

Install sqlx cli to enable rust to compile without a database.  This needs to be done after creating the tables so it can compile the binary and extract the metadata.
`cargo install --version 0.6.3 sqlx-cli`
`cargo sqlx prepare`
This creates the sqlx-data.json which allows the binary to be recompiled without the actual database existing.