resource "azurerm_kubernetes_cluster" "cluster" {
  name                = "wasmdemo"
  location            = "Germany West Central"
  resource_group_name = "wasmdemo"
  dns_prefix          = "wasmdemo"

  default_node_pool {
    name       = "default"
    node_count = "2"
    vm_size    = "standard_d2_v2"
  }

  identity {
    type = "SystemAssigned"
  }
}
