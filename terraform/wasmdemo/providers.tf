terraform {
  required_version = ">=1.6.0"
  required_providers {
    azurerm = {
      source  = "hashicorp/azurerm"
      version = "~>3.80"
    }
    random = {
      source  = "hashicorp/random"
      version = "~>3.6.0"
    }
  }
#   backend "azurerm" {
#     resource_group_name  = "wasmdemo"
#     storage_account_name = "wasmdemo"
#     container_name       = "wasmdemo"
#     key                  = "terraform.tfstate"
#   }
}

provider "azurerm" {
  features {}
}
