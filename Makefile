install: install-completion
	cargo install --path .
install-frozen: install-completion
	cargo install --frozen --path .
install-completion:
	mkdir --parents ~/.local/share/bash-completion/completions
	cp completion.sh ~/.local/share/bash-completion/completions/tangl
test:
	cargo llvm-cov

example:
	mkdir -p $(PWD)/target/example/construction-site-example
	docker compose up -d && docker compose down
	docker compose run --rm -t example bash
	rm -rf target/example

clean:
	rm ~/.cargo/bin/tangl
	rm ~/.local/share/bash-completion/completions/tangl