resource "azurerm_resource_group" "main" {
  name     = var.rgname_tfstate
  location = var.location
  tags     = merge(var.default_tags)
}

resource "azurerm_storage_account" "wasmdemo" {
  name                            = "wasmdemo"
  resource_group_name             = azurerm_resource_group.main.name
  location                        = azurerm_resource_group.main.location
  account_tier                    = var.account_tier
  account_kind                    = var.account_kind
  access_tier                     = var.access_tier
  account_replication_type        = var.account_replication_type
  min_tls_version                 = var.min_tls_version
  enable_https_traffic_only       = var.enable_https_traffic_only
  allow_nested_items_to_be_public = var.allow_nested_items_to_be_public

  blob_properties {
    last_access_time_enabled = var.last_access_time_enabled
  }

  tags = merge(var.default_tags)
}

resource "azurerm_storage_container" "tfstate" {
  name                  = "wasmdemo"
  storage_account_name  = azurerm_storage_account.wasmdemo.name
  container_access_type = var.container_access_type
}
