##### Install Configmap
`k create -f configmaps-liquibase-changelog.yml` 

##### Install Posgres
`helm install posgres -n postgres oci://registry-1.docker.io/bitnamicharts/postgresql -f values.yml`

##### Execute Liquibase
`k create -f liquibase-job.yml`
