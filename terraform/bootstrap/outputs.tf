output "primary_subscription" {
  description = "Displays the current subscription"
  value       = data.azurerm_subscription.primary.display_name
}

output "backend_storage_account" {
  description = "Displays the name of the current storage account used as a backend"
  value       = azurerm_storage_account.wasmdemo.name
}

output "backend_resource_group" {
  description = "Displays the name of the resource group, where the backend is located"
  value       = azurerm_resource_group.main.name
}

output "managed_by_terraform" {
  description = "Displays the name of the tag, which is used to identify resources managed by terraform"
  value       = var.default_tags["managed_by_terraform"]
}
