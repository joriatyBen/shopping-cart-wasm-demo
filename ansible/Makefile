.PHONY: prepare kepler metrics postgres spin-kube

prepare:
	ansible-galaxy collection install -r ansible-requirements.yml
	ansible-galaxy install -r ansible-requirements.yml
	pip install -r python-requirements.txt > /dev/null 2>&1


cert-manager: prepare
	ansible-playbook deploy-cert-manager.yml

prometheus: prepare
	ansible-playbook deploy-prometheus.yml

kepler: prepare
	ansible-playbook deploy-kepler.yml

postgres: prepare
	ansible-playbook deploy-postgres.yml

spin-kube: prepare
	ansible-playbook deploy-spin-kube.yml

carts-java: prepare
	ansible-playbook deploy-carts-java.yml

traefik: prepare
	ansible-playbook deploy-traefik.yml
