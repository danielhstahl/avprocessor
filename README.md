cargo run 
curl -X PUT http://127.0.0.1:8000/config -H 'Content-Type: application/json' -d @example_json.json

curl -X GET http://127.0.0.1:8000/config/latest

../../third_party/camilla/camilladsp -v -p1234 test.yaml




#plotcamillaconf test.yml