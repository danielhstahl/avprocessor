camilladsp -v -p1234 test.yaml

curl -X PUT http://127.0.0.1:8000/config -H 'Content-Type: application/json' -d @example_json.json

curl -X GET http://127.0.0.1:8000/config/latest

../../third_party/camilla/camilladsp -v -p1234 test.yaml

TODO:
* Add Gain filters for setting speaker volume (can gains be fractions or just integers?)
* Add Crossovers for the subwoofers so they don't get full signal