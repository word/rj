file { "/tmp/puppet_testfile":
  # use a fact from stdlib module to test module installation worked.
  content => $root_home,
}
