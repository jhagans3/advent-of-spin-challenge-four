#hurl --error-format long --variable serviceUrl="https://advent-of-spin-challenge-four-cawszfsk.fermyon.app" submit.hurl

POST https://advent-of-spin2023.fermyon.app/4
{
  "serviceUrl": "{{serviceUrl}}"
}

HTTP 200
