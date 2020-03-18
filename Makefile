.PHONY: release release_vagrant test test_trace test_nocapture clean up

release:
	$(info => Building a release binary)
	cargo build --release

release_vagrant:
	$(info => Building a release binary in Vagrant)
	vagrant rsync
	vagrant ssh -c 'cd /vagrant && cargo build --release'
	mkdir -p target/release
	vagrant scp default:/vagrant/target/release/rj target/release/

up:
	$(info => Bringing up vagrant box)
	vagrant up

test:
	$(info => Running tests)
	vagrant ssh -c 'cd /vagrant && sudo cargo test'

test_trace:
	$(info => Running tests with RUST_BACKTRACE)
	vagrant ssh -c 'cd /vagrant && sudo RUST_BACKTRACE=1 cargo test'

test_nocapture:
	$(info => Running tests with RUST_BACKTRACE)
	vagrant ssh -c 'cd /vagrant && sudo RUST_BACKTRACE=1 cargo test -- --nocapture'

fmt:
	$(info => Formatting)
	cargo fmt
	# cargo +nightly fmt

clean:
	$(info => Cleaning)
	rm -rf target/
