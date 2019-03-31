simple_map /* Inline comment */ { // Comment
  foo   = "bar"
  bar   = "baz"
  index = 1
}

/* Test Map */
simple_map {
  foo   = "bar"
  bar   = "baz"
  index = 0
}

// This is a useless security group
resource "security/group" "foobar" {
  name = "foobar" # Comment

  allow {
    name = "localhost" // Seems pointless
    cidrs = ["127.0.0.1/32"]
  }

  allow {
    name = "lan" /* Is this all our LAN CIDR? */
    cidrs = ["192.168.0.0/16"]
  }

  deny {
    # Now this is pointless
    name = "internet"
    cidrs = ["0.0.0.0/0"]
  }
}

# Might as well be this
resource "security/group" "second" {
  name = "second"

  allow {
    name = "all"
    cidrs = ["0.0.0.0/0"]
  }
}

// Instance
resource "instance" "an_instance" {
  name = "an_instance"
  image = "ubuntu:18.04"

  user "test" {
    root = true
  }
}
