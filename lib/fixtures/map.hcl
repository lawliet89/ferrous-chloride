simple_map /* Inline comment */ { // Comment
  foo   = "bar"
  bar   = "baz"
  index = 1
}

simple_map {
  foo   = "bar"
  bar   = "baz"
  index = 0
}

resource "security/group" "foobar" {
  name = "foobar"

  allow {
    name = "localhost"
    cidrs = ["127.0.0.1/32"]
  }

  allow {
    name = "lan"
    cidrs = ["192.168.0.0/16"]
  }

  deny {
    name = "internet"
    cidrs = ["0.0.0.0/0"]
  }
}

resource "security/group" "second" {
  name = "second"

  allow {
    name = "all"
    cidrs = ["0.0.0.0/0"]
  }
}

resource "instance" "an_instance" {
  name = "an_instance"
  image = "ubuntu:18.04"

  user "test" {
    root = true
  }
}
