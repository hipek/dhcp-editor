.PHONY: start build stop logs

start:
	docker compose up -d --build

build:
	docker compose build

stop:
	docker compose down

logs:
	docker compose logs -f
