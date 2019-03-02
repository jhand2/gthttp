all:
	cargo build

arpspoofr:
	cargo build arpspoofr

shijackr:
	cargo build shijackr

docker:
	docker build -t gthttp .
