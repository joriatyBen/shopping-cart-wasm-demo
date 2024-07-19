variable "location" {
  description = "(Required) The location where the resource should be created."
  type        = string
  default     = "Germany West Central"
}

variable "default_tags" {
  description = "Map of default tags, used as standard for the whole project."
  type        = any
  default = {
    managed_by_terraform = "True"
  }
}

variable "rgname_tfstate" {
  description = "(Required) The Name which should be used for the Resource Group. Changing this forces a new Resource Group to be created."
  type        = string
  default     = "wasmdemo"
}

variable "account_tier" {
  description = "(Required) Defines the Tier to use for this storage account. Valid options are Standard and Premium. For BlockBlobStorage and FileStorage accounts only Premium is valid. Changing this forces a new resource to be created."
  type        = string
  default     = "Standard"

  validation {
    condition = contains(["Standard", "Premium"], var.account_tier)
    error_message = "The value of the account_tier property is invalid. Only Standard and Premium are allowed"
  }
}

variable "account_kind" {
  description = "(Optional) Defines the Kind of account. Valid options are BlobStorage, BlockBlobStorage, FileStorage, Storage and StorageV2. Defaults to StorageV2."
  type        = string
  default     = "StorageV2"

  validation {
    condition = contains(["BlobStorage", "BlockBlobStorage", "FileStorage", "StorageV2"], var.account_kind)
    error_message = "The value of the account_kind property is invalid. Only BlobStorage, BlockBlobStorage, FileStorage and StorageV2 are allowed"
  }
}

variable "access_tier" {
  description = "(Optional) Defines the access tier for BlobStorage, FileStorage and StorageV2 accounts. Valid options are Hot and Cool, defaults to Hot."
  type        = string
  default     = "Cool"

  validation {
    condition = contains(["Hot", "Cool"], var.access_tier)
    error_message = "The value of the access_tier property is invalid. Only Hot and Cool are allowed"
  }
}

variable "account_replication_type" {
  description = "(Required) Defines the type of replication to use for this storage account. Valid options are LRS, GRS, RAGRS, ZRS, GZRS and RAGZRS."
  type        = string
  default     = "LRS"

  validation {
    condition = contains(["LRS", "GRS", "RAGRS", "ZRS", "GZRS", "RAGZRS"], var.account_replication_type)
    error_message = "The value of the account_replication_type property is invalid. Only LRS, GRS, RAGRS, ZRS, GZRS and RAGZRS are allowed"
  }
}

variable "min_tls_version" {
  description = "(Optional) The minimum supported TLS version for the storage account. Possible values are TLS1_0, TLS1_1, and TLS1_2. Defaults to TLS1_2 for new storage accounts."
  type        = string
  default     = "TLS1_2"

  validation {
    condition = contains(["TLS1_2"], var.min_tls_version)
    error_message = "The value of the account_replication_type property is invalid. Only TLS1_2 is allowed, because versions 1.0 and 1.1 are no longer supported by Microsoft."
  }
}

variable "enable_https_traffic_only" {
  description = "(Optional) Boolean flag which forces HTTPS if enabled, see here for more information. Defaults to true."
  type        = bool
  default     = true
}

variable "allow_nested_items_to_be_public" {
  description = "(Optional) Allow or disallow nested items within this Account to opt into being public. Defaults to true."
  type        = bool
  default     = false
}

variable "last_access_time_enabled" {
  description = "(Optional) Is the last access time based tracking enabled? Default to false."
  type        = bool
  default     = true
}

variable "default_action" {
  description = "(Required) Specifies the default action of allow or deny when no other rules match. Valid options are Deny or Allow."
  type        = string
  default     = "Allow"
}

variable "ip_rules" {
  description = "(Optional) List of public IP or IP ranges in CIDR Format. Only IPv4 addresses are allowed. /31 CIDRs, /32 CIDRs, and Private IP address ranges (as defined in RFC 1918), are not allowed."
  type = list(string)
  default = ["100.0.0.1"]
}

variable "container_access_type" {
  description = "(Optional) The access type for the container. Possible values are blob, container or private. Defaults to private."
  type        = string
  default     = "private"

  validation {
    condition = contains(["private"], var.container_access_type)
    error_message = "The value of the container_access_type property is invalid. Only private is allowed."
  }
}