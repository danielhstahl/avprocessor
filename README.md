camilladsp -v -p1234 test.yaml

curl -X PUT http://127.0.0.1:8000/config -H 'Content-Type: application/json' -d @example_json.json

curl -X GET http://127.0.0.1:8000/config/latest

../../third_party/camilla/camilladsp -v -p1234 test.yaml

TODO:
* Refactor to look like test.yml.  This includes adding gain to the final outputs
* Gains can be f32 (not i32)



plotcamillaconf test.yml