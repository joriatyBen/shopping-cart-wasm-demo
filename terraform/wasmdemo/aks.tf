resource "azurerm_kubernetes_cluster" "cluster" {
  name                = "wasmdemo"
  location            = "Germany West Central"
  resource_group_name = "wasmdemo"
  dns_prefix          = "wasmdemo"
  http_application_routing_enabled = true

  default_node_pool {
    name       = "default"
    node_count = "2"
    vm_size    = "standard_d2_v2"
    enable_auto_scaling = true
    max_count = "4"
    min_count = "2"
    type = "VirtualMachineScaleSets"
  }

  identity {
    type = "SystemAssigned"
  }
}
