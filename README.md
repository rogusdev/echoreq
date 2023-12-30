# echoreq
Simple webserver to echo http requests -- headers, etc, including multipart form-data (binary converted to printable chars). For testing!

Install it: `cargo install echoreq`, then run it `echoreq` (default port is 3000, change with `PORT=5000 echoreq` etc), then send requests to it. Examples:

```
curl -X POST http://localhost:3000/hello/name \
    -b tower.sid=abcd1234 -d "param1=value1&param2=value2"

curl -X POST http://localhost:3000/echo/post/json \
    -H "Content-Type: application/json" \
    -d '{"productId": 123456, "quantity": 100}'

curl -X POST http://localhost:3000/form-data/text \
    -F title='Cool story' -F year=2023 -F thumb=@demo.txt

curl -X POST http://localhost:3000/form-data/image \
    -F title='Cool story' -F year=2023 -F thumb=@demo.png
```

Sample response:
```
POST /echo/post/json
host: localhost:3000
user-agent: curl/7.81.0
accept: */*
content-type: application/json
content-length: 38

{"productId": 123456, "quantity": 100}
```

(Curl will also add a `⏎` at the end to indicate that there was no final newline in the response, because `echoreq` echoes the request body as it is received, including any trailing newlines or not. But this curl addition is not part of the response, as is demonstrated in the tests.)

It is also appropriate to git clone this repo and add your own tests (locally) to verify requests ala the various tests that reproduce the above examples. Testing is the real purpose of this application!
