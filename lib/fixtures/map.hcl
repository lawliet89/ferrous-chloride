simple_map {
  foo = "bar"
  bar = "baz"
}

simple_map {
  foo = "bar"
  bar = "baz"
}

resource "security/group" "foobar" {
  allow {
    cidrs = ["127.0.0.1/32"]
  }

  deny {
    cidrs = ["0.0.0.0/0"]
  }
}
