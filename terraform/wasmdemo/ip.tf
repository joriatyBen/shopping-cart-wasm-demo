data "azurerm_subscription" "primary" {
}

data "azurerm_client_config" "example" {
}

resource "azurerm_public_ip" "example" {
  name                = "wasmdemoip"
  resource_group_name = "wasmdemo"
  location            = "Germany West Central"
  allocation_method   = "Static"
  sku = "Standard"
}

resource "azurerm_role_assignment" "example" {
  scope                = "/subscriptions/27e62b89-4b81-41f9-a527-41f23430ffca/resourceGroups/wasmdemo"
  role_definition_name = "Network Contributor"
  principal_id         = "b22d65ae-70d9-48bb-9931-8dfd2346e571"
}