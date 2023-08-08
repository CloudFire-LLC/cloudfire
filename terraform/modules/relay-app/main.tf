locals {
  application_name    = var.application_name != null ? var.application_name : var.image
  application_version = var.application_version != null ? var.application_version : var.image_tag

  application_labels = merge({
    managed_by  = "terraform"
    application = local.application_name
    version     = local.application_version
  }, var.application_labels)

  google_health_check_ip_ranges = [
    "130.211.0.0/22",
    "35.191.0.0/16"
  ]

  environment_variables = concat([
    {
      name  = "LISTEN_ADDRESS_DISCOVERY_METHOD"
      value = "gce_metadata"
    },
    {
      name  = "RUST_LOG"
      value = var.observability_log_level
    },
    {
      name  = "JSON_LOG"
      value = "true"
    },
    {
      name  = "METRICS_ADDR"
      value = "0.0.0.0:8080"
    },
    {
      name  = "PORTAL_TOKEN"
      value = var.portal_token
    },
    {
      name  = "PORTAL_WS_URL"
      value = var.portal_websocket_url
    }
  ], var.application_environment_variables)
}

# Fetch most recent COS image
data "google_compute_image" "coreos" {
  family  = "cos-105-lts"
  project = "cos-cloud"
}

# Create IAM role for the application instances
resource "google_service_account" "application" {
  project = var.project_id

  account_id   = "app-${local.application_name}"
  display_name = "${local.application_name} app"
  description  = "Service account for ${local.application_name} application instances."
}

## Allow application service account to pull images from the container registry
resource "google_project_iam_member" "artifacts" {
  project = var.project_id

  role = "roles/artifactregistry.reader"

  member = "serviceAccount:${google_service_account.application.email}"
}

## Allow fluentbit to injest logs
resource "google_project_iam_member" "logs" {
  project = var.project_id

  role = "roles/logging.logWriter"

  member = "serviceAccount:${google_service_account.application.email}"
}

## Allow reporting application errors
resource "google_project_iam_member" "errors" {
  project = var.project_id

  role = "roles/errorreporting.writer"

  member = "serviceAccount:${google_service_account.application.email}"
}

## Allow reporting metrics
resource "google_project_iam_member" "metrics" {
  project = var.project_id

  role = "roles/monitoring.metricWriter"

  member = "serviceAccount:${google_service_account.application.email}"
}

## Allow reporting metrics
resource "google_project_iam_member" "service_management" {
  project = var.project_id

  role = "roles/servicemanagement.reporter"

  member = "serviceAccount:${google_service_account.application.email}"
}

## Allow appending traces
resource "google_project_iam_member" "cloudtrace" {
  project = var.project_id

  role = "roles/cloudtrace.agent"

  member = "serviceAccount:${google_service_account.application.email}"
}

resource "google_compute_instance_template" "application" {
  for_each = var.instances

  project = var.project_id

  name_prefix = "${local.application_name}-${each.key}-"

  description = "This template is used to create ${local.application_name} instances."

  machine_type = each.value.type

  can_ip_forward = false

  tags = ["app-${local.application_name}"]

  labels = merge({
    container-vm = data.google_compute_image.coreos.name
  }, local.application_labels)

  scheduling {
    automatic_restart   = true
    on_host_maintenance = "MIGRATE"
    provisioning_model  = "STANDARD"
  }

  disk {
    source_image = data.google_compute_image.coreos.self_link
    auto_delete  = true
    boot         = true
  }

  network_interface {
    network = var.vpc_network

    access_config {
      network_tier = "PREMIUM"
      # Ephimerical IP address
    }
  }

  service_account {
    email = google_service_account.application.email

    scopes = [
      # Those are copying gke-default scopes
      "storage-ro",
      "logging-write",
      "monitoring",
      "service-management",
      "service-control",
      "trace",
      # Required to discover the other instances in the Erlang Cluster
      "compute-ro",
    ]
  }

  metadata = merge({
    gce-container-declaration = yamlencode({
      spec = {
        containers = [{
          name  = local.application_name != null ? local.application_name : var.image
          image = "${var.container_registry}/${var.image_repo}/${var.image}:${var.image_tag}"
          env   = local.environment_variables
        }]

        volumes = []

        restartPolicy = "Always"
      }
    })

    # Enable FluentBit agent for logging, which will be default one from COS 109
    google-logging-enabled       = "true"
    google-logging-use-fluentbit = "true"

    # Report health-related metrics to Cloud Monitoring
    google-monitoring-enabled = "true"
  })

  depends_on = [
    google_project_service.compute,
    google_project_service.pubsub,
    google_project_service.bigquery,
    google_project_service.container,
    google_project_service.stackdriver,
    google_project_service.logging,
    google_project_service.monitoring,
    google_project_service.cloudprofiler,
    google_project_service.cloudtrace,
    google_project_service.servicenetworking,
    google_project_iam_member.artifacts,
    google_project_iam_member.logs,
    google_project_iam_member.errors,
    google_project_iam_member.metrics,
    google_project_iam_member.service_management,
    google_project_iam_member.cloudtrace,
  ]

  lifecycle {
    create_before_destroy = true
  }
}

# Create health checks for the application ports
resource "google_compute_health_check" "port" {
  project = var.project_id

  name = "${local.application_name}-${var.health_check.name}"

  check_interval_sec  = var.health_check.check_interval_sec != null ? var.health_check.check_interval_sec : 5
  timeout_sec         = var.health_check.timeout_sec != null ? var.health_check.timeout_sec : 5
  healthy_threshold   = var.health_check.healthy_threshold != null ? var.health_check.healthy_threshold : 2
  unhealthy_threshold = var.health_check.unhealthy_threshold != null ? var.health_check.unhealthy_threshold : 2

  log_config {
    enable = false
  }

  http_health_check {
    port = var.health_check.port

    host         = var.health_check.http_health_check.host
    request_path = var.health_check.http_health_check.request_path
    response     = var.health_check.http_health_check.response
  }
}

# Use template to deploy zonal instance group
resource "google_compute_region_instance_group_manager" "application" {
  for_each = var.instances

  project = var.project_id

  name = "${local.application_name}-group-${each.key}"

  base_instance_name = local.application_name

  region                    = each.key
  distribution_policy_zones = each.value.zones

  target_size = each.value.replicas

  wait_for_instances        = true
  wait_for_instances_status = "STABLE"

  version {
    instance_template = google_compute_instance_template.application[each.key].self_link
  }

  named_port {
    name = "stun"
    port = 3478
  }

  auto_healing_policies {
    initial_delay_sec = var.health_check.initial_delay_sec

    health_check = google_compute_health_check.port.self_link
  }

  update_policy {
    type           = "PROACTIVE"
    minimal_action = "REPLACE"

    max_unavailable_fixed = 0
    max_surge_fixed       = max(length(each.value.zones), each.value.replicas - 1)
  }

  depends_on = [
    google_compute_instance_template.application
  ]
}

# Define a security policy which allows to filter traffic by IP address,
# an edge security policy can also detect and block common types of web attacks
resource "google_compute_security_policy" "default" {
  project = var.project_id

  name = local.application_name

  rule {
    action   = "allow"
    priority = "2147483647"

    match {
      versioned_expr = "SRC_IPS_V1"

      config {
        src_ip_ranges = ["*"]
      }
    }

    description = "default allow rule"
  }
}

# Open ports for the web
resource "google_compute_firewall" "stun-turn" {
  project = var.project_id

  name    = "${local.application_name}-firewall-lb-to-instances"
  network = var.vpc_network

  source_ranges = ["0.0.0.0/0"]
  target_tags   = ["app-${local.application_name}"]

  allow {
    protocol = "tcp"
    ports    = ["3478", "49152-65535"]
  }

  allow {
    protocol = "udp"
    ports    = ["3478", "49152-65535"]
  }
}

## Open metrics port for the health checks
resource "google_compute_firewall" "http-health-checks" {
  project = var.project_id

  name    = "${local.application_name}-healthcheck"
  network = var.vpc_network

  source_ranges = local.google_health_check_ip_ranges
  target_tags   = ["app-${local.application_name}"]

  allow {
    protocol = var.health_check.protocol
    ports    = [var.health_check.port]
  }
}

# Allow outbound traffic
resource "google_compute_firewall" "egress-ipv4" {
  project = var.project_id

  name      = "${local.application_name}-egress-ipv4"
  network   = var.vpc_network
  direction = "EGRESS"

  target_tags        = ["app-${local.application_name}"]
  destination_ranges = ["0.0.0.0/0"]

  allow {
    protocol = "udp"
  }
}

resource "google_compute_firewall" "egress-ipv6" {
  project = var.project_id

  name      = "${local.application_name}-egress-ipv6"
  network   = var.vpc_network
  direction = "EGRESS"

  target_tags        = ["app-${local.application_name}"]
  destination_ranges = ["::/0"]

  allow {
    protocol = "udp"
  }
}
