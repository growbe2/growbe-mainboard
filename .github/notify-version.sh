#! /bin/bash
#
export VERSION=$(./target/release/growbe-mainboard version)

curl --location --request POST 'https://api.growbe.ca/growbe-mainboard/version' --header "Authorization: Bearer ${TOKEN}" --header 'Content-Type: application/json' --data-raw "{ \"name\": \"${VERSION}\" }"
