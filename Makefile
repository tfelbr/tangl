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
	docker compose build
	docker run -it --rm tangl:1

clean:
	rm ~/.cargo/bin/tangl
	rm ~/.local/share/bash-completion/completions/tangl