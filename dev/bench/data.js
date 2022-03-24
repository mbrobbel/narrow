window.BENCHMARK_DATA = {
  "lastUpdate": 1648159896757,
  "repoUrl": "https://github.com/mbrobbel/narrow",
  "entries": {
    "narrow": [
      {
        "commit": {
          "author": {
            "email": "m1brobbel@gmail.com",
            "name": "Matthijs Brobbel",
            "username": "mbrobbel"
          },
          "committer": {
            "email": "m1brobbel@gmail.com",
            "name": "Matthijs Brobbel",
            "username": "mbrobbel"
          },
          "distinct": true,
          "id": "38c591c11498200ef2b621ed4c967b94b17292bd",
          "message": "Update bench workflow",
          "timestamp": "2022-03-24T23:07:00+01:00",
          "tree_id": "da511625fb19f7b4db851a62577f041b817f5282",
          "url": "https://github.com/mbrobbel/narrow/commit/38c591c11498200ef2b621ed4c967b94b17292bd"
        },
        "date": 1648159896190,
        "tool": "cargo",
        "benches": [
          {
            "name": "Bitmap::from_iter/narrow/12345/0",
            "value": 8512,
            "range": "± 163",
            "unit": "ns/iter"
          },
          {
            "name": "Bitmap::from_iter/narrow/12345/0.5",
            "value": 8788,
            "range": "± 112",
            "unit": "ns/iter"
          },
          {
            "name": "Bitmap::from_iter/narrow/12345/1",
            "value": 8560,
            "range": "± 229",
            "unit": "ns/iter"
          },
          {
            "name": "Bitmap::into_iter/narrow/12345/0",
            "value": 15257,
            "range": "± 260",
            "unit": "ns/iter"
          },
          {
            "name": "Bitmap::into_iter/narrow/12345/0.5",
            "value": 15100,
            "range": "± 336",
            "unit": "ns/iter"
          },
          {
            "name": "Bitmap::into_iter/narrow/12345/1",
            "value": 15353,
            "range": "± 387",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}