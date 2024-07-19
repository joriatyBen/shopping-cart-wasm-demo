terraform {
  backend "azurerm" {
    resource_group_name  = "wasmdemo"
    storage_account_name = "wasmdemo"
    container_name       = "wasmdemo"
    key                  = "terraform.tfstate"
  }
}