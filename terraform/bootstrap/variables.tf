variable "location" {
  type        = string
  default     = "Germany West Central"
}

variable "default_tags" {
  type        = any
  default = {
    managed_by_terraform = "True"
  }
}

variable "rgname_tfstate" {
  type        = string
  default     = "wasmdemo"
}

variable "account_tier" {
  type        = string
  default     = "Standard"

  validation {
    condition = contains(["Standard", "Premium"], var.account_tier)
    error_message = "The value of the account_tier property is invalid. Only Standard and Premium are allowed"
  }
}

variable "account_kind" {
  type        = string
  default     = "StorageV2"

  validation {
    condition = contains(["BlobStorage", "BlockBlobStorage", "FileStorage", "StorageV2"], var.account_kind)
    error_message = "The value of the account_kind property is invalid. Only BlobStorage, BlockBlobStorage, FileStorage and StorageV2 are allowed"
  }
}

variable "access_tier" {
  type        = string
  default     = "Cool"

  validation {
    condition = contains(["Hot", "Cool"], var.access_tier)
    error_message = "The value of the access_tier property is invalid. Only Hot and Cool are allowed"
  }
}

variable "account_replication_type" {
  type        = string
  default     = "LRS"

  validation {
    condition = contains(["LRS", "GRS", "RAGRS", "ZRS", "GZRS", "RAGZRS"], var.account_replication_type)
    error_message = "The value of the account_replication_type property is invalid. Only LRS, GRS, RAGRS, ZRS, GZRS and RAGZRS are allowed"
  }
}

variable "min_tls_version" {
  type        = string
  default     = "TLS1_2"

  validation {
    condition = contains(["TLS1_2"], var.min_tls_version)
    error_message = "The value of the account_replication_type property is invalid. Only TLS1_2 is allowed, because versions 1.0 and 1.1 are no longer supported by Microsoft."
  }
}

variable "enable_https_traffic_only" {
  type        = bool
  default     = true
}

variable "allow_nested_items_to_be_public" {
  type        = bool
  default     = false
}

variable "last_access_time_enabled" {
  type        = bool
  default     = true
}

variable "default_action" {
  type        = string
  default     = "Allow"
}

variable "ip_rules" {
  type = list(string)
  default = ["100.0.0.1"]
}

variable "container_access_type" {
  type        = string
  default     = "private"

  validation {
    condition = contains(["private"], var.container_access_type)
    error_message = "The value of the container_access_type property is invalid. Only private is allowed."
  }
}