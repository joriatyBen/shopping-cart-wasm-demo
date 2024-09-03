output "primary_subscription" {
  value       = data.azurerm_subscription.primary.display_name
}

output "backend_storage_account" {
  value       = azurerm_storage_account.wasmdemo.name
}

output "backend_resource_group" {
  value       = azurerm_resource_group.main.name
}

output "managed_by_terraform" {
  value       = var.default_tags["managed_by_terraform"]
}
