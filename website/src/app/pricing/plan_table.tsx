"use client";

import { Tooltip } from "flowbite-react";
import { HiCheck } from "react-icons/hi2";
import { useEffect } from "react";
import { initFlowbite } from "flowbite";
import Link from "next/link";

export default function PlanTable() {
  useEffect(() => {
    // Manually init flowbite's data-toggle listeners since we're using custom components
    initFlowbite();
  }, []);
  return (
    <table className="w-full mx-auto text-left table-fixed max-w-screen-sm sm:max-w-screen-md">
      <thead>
        <tr>
          <th scope="col"></th>
          <th
            scope="col"
            className="mx-1 py-6 uppercase text-primary-450 text-center tracking-light"
          >
            Starter
          </th>
          <th
            scope="col"
            className="mx-1 py-6 uppercase text-primary-450 text-center tracking-light"
          >
            Team
          </th>
          <th
            scope="col"
            className="mx-1 py-6 uppercase text-primary-450 text-center tracking-light"
          >
            Enterprise
          </th>
        </tr>
      </thead>
      <tbody>
        <tr className="border-b border-1 border-neutral-200">
          <td className="gmx-1 py-4">
            <span
              data-tooltip-target="users-tooltip"
              data-tooltip-placement="top"
              className="underline hover:no-underline cursor-help"
            >
              Users
            </span>
            <div
              id="users-tooltip"
              role="tooltip"
              className="text-wrap absolute z-10 invisible inline-block px-3 py-2 text-xs font-medium text-white transition-opacity duration-100 bg-neutral-900 rounded shadow-sm opacity-90 tooltip"
            >
              Includes both admins and end-users of your Firezone account
              <div className="tooltip-arrow" data-popper-arrow></div>
            </div>
          </td>
          <td className="gmx-1 py-4 text-center">6</td>
          <td className="gmx-1 py-4 text-center">100</td>
          <td className="gmx-1 py-4 text-center">No limit</td>
        </tr>
        <tr className="border-b border-1 border-neutral-200">
          <td className="gmx-1 py-4">
            <span
              data-tooltip-target="sa-tooltip"
              data-tooltip-placement="top"
              className="underline hover:no-underline cursor-help"
            >
              Service Accounts
            </span>
            <div
              id="sa-tooltip"
              role="tooltip"
              className="text-wrap absolute z-10 invisible inline-block px-3 py-2 text-xs font-medium text-white transition-opacity duration-100 bg-neutral-900 rounded shadow-sm opacity-90 tooltip"
            >
              Machine accounts used to access Resources without a user present
              <div className="tooltip-arrow" data-popper-arrow></div>
            </div>
          </td>
          <td className="gmx-1 py-4 text-center">10</td>
          <td className="gmx-1 py-4 text-center">100</td>
          <td className="gmx-1 py-4 text-center">No limit</td>
        </tr>
        <tr className="border-b border-1 border-neutral-200">
          <td className="gmx-1 py-4">
            <span
              data-tooltip-target="sites-tooltip"
              data-tooltip-placement="top"
              className="underline hover:no-underline cursor-help"
            >
              Sites
            </span>
            <div
              id="sites-tooltip"
              role="tooltip"
              className="text-wrap absolute z-10 invisible inline-block px-3 py-2 text-xs font-medium text-white transition-opacity duration-100 bg-neutral-900 rounded shadow-sm opacity-90 tooltip"
            >
              Sites are a collection of Gateways and Resources that share the
              same network connectivity context. Typically a subnet or VPC.
              <div className="tooltip-arrow" data-popper-arrow></div>
            </div>
          </td>
          <td className="gmx-1 py-4 text-center">10</td>
          <td className="gmx-1 py-4 text-center">100</td>
          <td className="gmx-1 py-4 text-center">No limit</td>
        </tr>
        <tr className="border-b border-1 border-neutral-200">
          <td className="gmx-1 py-4">
            <span
              data-tooltip-target="admins-tooltip"
              data-tooltip-placement="top"
              className="underline hover:no-underline cursor-help"
            >
              Admins
            </span>
            <div
              id="admins-tooltip"
              role="tooltip"
              className="text-wrap absolute z-10 invisible inline-block px-3 py-2 text-xs font-medium text-white transition-opacity duration-100 bg-neutral-900 rounded shadow-sm opacity-90 tooltip"
            >
              Users with account-wide access to deploy Gateways, manage billing,
              and edit users, Sites, or other configuration
              <div className="tooltip-arrow" data-popper-arrow></div>
            </div>
          </td>
          <td className="gmx-1 py-4 text-center">1</td>
          <td className="gmx-1 py-4 text-center">10</td>
          <td className="gmx-1 py-4 text-center">No limit</td>
        </tr>
        <tr className="border-b border-1 border-neutral-200">
          <td className="gmx-1 py-4">
            <span
              data-tooltip-target="policies-tooltip"
              data-tooltip-placement="top"
              className="underline hover:no-underline cursor-help"
            >
              Policies
            </span>
            <div
              id="policies-tooltip"
              role="tooltip"
              className="text-wrap absolute z-10 invisible inline-block px-3 py-2 text-xs font-medium text-white transition-opacity duration-100 bg-neutral-900 rounded shadow-sm opacity-90 tooltip"
            >
              Policies control access to Resources (e.g. Group “A” may access
              Resource “B”)
              <div className="tooltip-arrow" data-popper-arrow></div>
            </div>
          </td>
          <td className="gmx-1 py-4 text-center">No limit</td>
          <td className="gmx-1 py-4 text-center">No limit</td>
          <td className="gmx-1 py-4 text-center">No limit</td>
        </tr>
        <tr className="border-b border-1 border-neutral-200">
          <td className="gmx-1 py-4">
            <span
              data-tooltip-target="resources-tooltip"
              data-tooltip-placement="top"
              className="underline hover:no-underline cursor-help"
            >
              Resources
            </span>
            <div
              id="resources-tooltip"
              role="tooltip"
              className="text-wrap absolute z-10 invisible inline-block px-3 py-2 text-xs font-medium text-white transition-opacity duration-100 bg-neutral-900 rounded shadow-sm opacity-90 tooltip"
            >
              Anything you wish to manage access to (e.g. database, VPC, home
              network, web server, SaaS application)
              <div className="tooltip-arrow" data-popper-arrow></div>
            </div>
          </td>
          <td className="gmx-1 py-4 text-center">No limit</td>
          <td className="gmx-1 py-4 text-center">No limit</td>
          <td className="gmx-1 py-4 text-center">No limit</td>
        </tr>
        <tr className="border-b border-1 border-neutral-200">
          <td className="gmx-1 py-4">
            <span
              data-tooltip-target="devices-tooltip"
              data-tooltip-placement="top"
              className="underline hover:no-underline cursor-help"
            >
              Connected Clients
            </span>
            <div
              id="devices-tooltip"
              role="tooltip"
              className="text-wrap absolute z-10 invisible inline-block px-3 py-2 text-xs font-medium text-white transition-opacity duration-100 bg-neutral-900 rounded shadow-sm opacity-90 tooltip"
            >
              Any device or machine that the Firezone Client connects from
              <div className="tooltip-arrow" data-popper-arrow></div>
            </div>
          </td>
          <td className="gmx-1 py-4 text-center">No limit</td>
          <td className="gmx-1 py-4 text-center">No limit</td>
          <td className="gmx-1 py-4 text-center">No limit</td>
        </tr>
        <tr>
          <td
            colSpan={4}
            className="gmx-1 pt-12 pb-4 text-lg uppercase font-semibold text-primary-450 tracking-light"
          >
            Networking Features
          </td>
        </tr>
        <tr className="border-b border-1 border-neutral-200">
          <td className="gmx-1 py-4">
            <span
              data-tooltip-target="nat-holepunching-tooltip"
              data-tooltip-placement="top"
              className="underline hover:no-underline cursor-help"
            >
              NAT hole punching
            </span>
            <div
              id="nat-holepunching-tooltip"
              role="tooltip"
              className="text-wrap absolute z-10 invisible inline-block px-3 py-2 text-xs font-medium text-white transition-opacity duration-100 bg-neutral-900 rounded shadow-sm opacity-90 tooltip"
            >
              Connect directly to Resources without opening inbound firewall
              ports
              <div className="tooltip-arrow" data-popper-arrow></div>
            </div>
          </td>
          <td className="gmx-1 py-4">
            <HiCheck className="mx-auto flex-shrink-0 w-5 h-5 text-neutral-900" />
          </td>
          <td className="gmx-1 py-4">
            <HiCheck className="mx-auto flex-shrink-0 w-5 h-5 text-neutral-900" />
          </td>
          <td className="gmx-1 py-4">
            <HiCheck className="mx-auto flex-shrink-0 w-5 h-5 text-neutral-900" />
          </td>
        </tr>
        <tr className="border-b border-1 border-neutral-200">
          <td className="gmx-1 py-4">
            <span
              data-tooltip-target="native-firezone-clients-tooltip"
              data-tooltip-placement="top"
              className="underline hover:no-underline cursor-help"
            >
              Native Firezone Clients
            </span>
            <div
              id="native-firezone-clients-tooltip"
              role="tooltip"
              className="text-wrap absolute z-10 invisible inline-block px-3 py-2 text-xs font-medium text-white transition-opacity duration-100 bg-neutral-900 rounded shadow-sm opacity-90 tooltip"
            >
              Native client apps for all major platforms
              <div className="tooltip-arrow" data-popper-arrow></div>
            </div>
          </td>
          <td className="gmx-1 py-4">
            <HiCheck className="mx-auto flex-shrink-0 w-5 h-5 text-neutral-900" />
          </td>
          <td className="gmx-1 py-4">
            <HiCheck className="mx-auto flex-shrink-0 w-5 h-5 text-neutral-900" />
          </td>
          <td className="gmx-1 py-4">
            <HiCheck className="mx-auto flex-shrink-0 w-5 h-5 text-neutral-900" />
          </td>
        </tr>
        <tr className="border-b border-1 border-neutral-200">
          <td className="gmx-1 py-4">
            <span
              data-tooltip-target="split-tunneling-tooltip"
              data-tooltip-placement="top"
              className="underline hover:no-underline cursor-help"
            >
              Split tunneling
            </span>
            <div
              id="split-tunneling-tooltip"
              role="tooltip"
              className="text-wrap absolute z-10 invisible inline-block px-3 py-2 text-xs font-medium text-white transition-opacity duration-100 bg-neutral-900 rounded shadow-sm opacity-90 tooltip"
            >
              Route traffic to Resources through Firezone leaving other traffic
              unaffected
              <div className="tooltip-arrow" data-popper-arrow></div>
            </div>
          </td>
          <td className="gmx-1 py-4">
            <HiCheck className="mx-auto flex-shrink-0 w-5 h-5 text-neutral-900" />
          </td>
          <td className="gmx-1 py-4">
            <HiCheck className="mx-auto flex-shrink-0 w-5 h-5 text-neutral-900" />
          </td>
          <td className="gmx-1 py-4">
            <HiCheck className="mx-auto flex-shrink-0 w-5 h-5 text-neutral-900" />
          </td>
        </tr>
        <tr className="border-b border-1 border-neutral-200">
          <td className="gmx-1 py-4">
            <span
              data-tooltip-target="ipv6-tooltip"
              data-tooltip-placement="top"
              className="underline hover:no-underline cursor-help"
            >
              IPv4 and IPv6 Resources
            </span>
            <div
              id="ipv6-tooltip"
              role="tooltip"
              className="text-wrap absolute z-10 invisible inline-block px-3 py-2 text-xs font-medium text-white transition-opacity duration-100 bg-neutral-900 rounded shadow-sm opacity-90 tooltip"
            >
              Connect to Resources over IPv4 or IPv6
              <div className="tooltip-arrow" data-popper-arrow></div>
            </div>
          </td>
          <td className="gmx-1 py-4">
            <HiCheck className="mx-auto flex-shrink-0 w-5 h-5 text-neutral-900" />
          </td>
          <td className="gmx-1 py-4">
            <HiCheck className="mx-auto flex-shrink-0 w-5 h-5 text-neutral-900" />
          </td>
          <td className="gmx-1 py-4">
            <HiCheck className="mx-auto flex-shrink-0 w-5 h-5 text-neutral-900" />
          </td>
        </tr>
        <tr className="border-b border-1 border-neutral-200">
          <td className="gmx-1 py-4">
            <span
              data-tooltip-target="nat64-tooltip"
              data-tooltip-placement="top"
              className="underline hover:no-underline cursor-help"
            >
              Automatic NAT64
            </span>
            <div
              id="nat64-tooltip"
              role="tooltip"
              className="text-wrap absolute z-10 invisible inline-block px-3 py-2 text-xs font-medium text-white transition-opacity duration-100 bg-neutral-900 rounded shadow-sm opacity-90 tooltip"
            >
              Connect to IPv6-only Resources from IPv4-only networks and
              vice-versa
              <div className="tooltip-arrow" data-popper-arrow></div>
            </div>
          </td>
          <td className="gmx-1 py-4">
            <HiCheck className="mx-auto flex-shrink-0 w-5 h-5 text-neutral-900" />
          </td>
          <td className="gmx-1 py-4">
            <HiCheck className="mx-auto flex-shrink-0 w-5 h-5 text-neutral-900" />
          </td>
          <td className="gmx-1 py-4">
            <HiCheck className="mx-auto flex-shrink-0 w-5 h-5 text-neutral-900" />
          </td>
        </tr>
        <tr className="border-b border-1 border-neutral-200">
          <td className="gmx-1 py-4">
            <span
              data-tooltip-target="dns-routing-tooltip"
              data-tooltip-placement="top"
              className="underline hover:no-underline cursor-help"
            >
              DNS-based routing
            </span>
            <div
              id="dns-routing-tooltip"
              role="tooltip"
              className="text-wrap absolute z-10 invisible inline-block px-3 py-2 text-xs font-medium text-white transition-opacity duration-100 bg-neutral-900 rounded shadow-sm opacity-90 tooltip"
            >
              Route traffic through Firezone based on DNS matching rules
              <div className="tooltip-arrow" data-popper-arrow></div>
            </div>
          </td>
          <td className="gmx-1 py-4">
            <HiCheck className="mx-auto flex-shrink-0 w-5 h-5 text-neutral-900" />
          </td>
          <td className="gmx-1 py-4">
            <HiCheck className="mx-auto flex-shrink-0 w-5 h-5 text-neutral-900" />
          </td>
          <td className="gmx-1 py-4">
            <HiCheck className="mx-auto flex-shrink-0 w-5 h-5 text-neutral-900" />
          </td>
        </tr>
        <tr className="border-b border-1 border-neutral-200">
          <td className="gmx-1 py-4">
            <span
              data-tooltip-target="gateway-load-balancing-tooltip"
              data-tooltip-placement="top"
              className="underline hover:no-underline cursor-help"
            >
              Gateway load-balancing
            </span>
            <div
              id="gateway-load-balancing-tooltip"
              role="tooltip"
              className="text-wrap absolute z-10 invisible inline-block px-3 py-2 text-xs font-medium text-white transition-opacity duration-100 bg-neutral-900 rounded shadow-sm opacity-90 tooltip"
            >
              Spread traffic across multiple Gateways within a Site
              <div className="tooltip-arrow" data-popper-arrow></div>
            </div>
          </td>
          <td className="gmx-1 py-4">
            <HiCheck className="mx-auto flex-shrink-0 w-5 h-5 text-neutral-900" />
          </td>
          <td className="gmx-1 py-4">
            <HiCheck className="mx-auto flex-shrink-0 w-5 h-5 text-neutral-900" />
          </td>
          <td className="gmx-1 py-4">
            <HiCheck className="mx-auto flex-shrink-0 w-5 h-5 text-neutral-900" />
          </td>
        </tr>
        <tr className="border-b border-1 border-neutral-200">
          <td className="gmx-1 py-4">
            <span
              data-tooltip-target="automatic-gateway-failover-tooltip"
              data-tooltip-placement="top"
              className="underline hover:no-underline cursor-help"
            >
              Automatic Gateway failover
            </span>
            <div
              id="automatic-gateway-failover-tooltip"
              role="tooltip"
              className="text-wrap absolute z-10 invisible inline-block px-3 py-2 text-xs font-medium text-white transition-opacity duration-100 bg-neutral-900 rounded shadow-sm opacity-90 tooltip"
            >
              Clients automatically switch from unhealthy Gateways to healthy
              ones
              <div className="tooltip-arrow" data-popper-arrow></div>
            </div>
          </td>
          <td className="gmx-1 py-4">
            <HiCheck className="mx-auto flex-shrink-0 w-5 h-5 text-neutral-900" />
          </td>
          <td className="gmx-1 py-4">
            <HiCheck className="mx-auto flex-shrink-0 w-5 h-5 text-neutral-900" />
          </td>
          <td className="gmx-1 py-4">
            <HiCheck className="mx-auto flex-shrink-0 w-5 h-5 text-neutral-900" />
          </td>
        </tr>
        <tr className="border-b border-1 border-neutral-200">
          <td className="gmx-1 py-4">
            <span
              data-tooltip-target="relays-tooltip"
              data-tooltip-placement="top"
              className="underline hover:no-underline cursor-help"
            >
              Global Relay network
            </span>
            <div
              id="relays-tooltip"
              role="tooltip"
              className="text-wrap absolute z-10 invisible inline-block px-3 py-2 text-xs font-medium text-white transition-opacity duration-100 bg-neutral-900 rounded shadow-sm opacity-90 tooltip"
            >
              Speed and availability of Firezone-managed relays that are used
              when a direct connection is not possible.
              <div className="tooltip-arrow" data-popper-arrow></div>
            </div>
          </td>
          <td className="gmx-1 py-4 flex-wrap text-center">Basic</td>
          <td className="gmx-1 py-4 flex-wrap text-center">Standard</td>
          <td className="gmx-1 py-4 flex-wrap text-center">Unthrottled</td>
        </tr>
        <tr>
          <td
            colSpan={4}
            className="mx-1 pt-12 pb-4 text-lg uppercase font-semibold text-primary-450 tracking-light"
          >
            Authentication & Authorization
          </td>
        </tr>
        <tr className="border-b border-1 border-neutral-200">
          <td className="gmx-1 py-4">
            <span
              data-tooltip-target="resource-level-access-policies-tooltip"
              data-tooltip-placement="top"
              className="underline hover:no-underline cursor-help"
            >
              Resource-level access policies
            </span>
            <div
              id="resource-level-access-policies-tooltip"
              role="tooltip"
              className="text-wrap absolute z-10 invisible inline-block px-3 py-2 text-xs font-medium text-white transition-opacity duration-100 bg-neutral-900 rounded shadow-sm opacity-90 tooltip"
            >
              Control access to Resources based on user identity and group
              <div className="tooltip-arrow" data-popper-arrow></div>
            </div>
          </td>
          <td className="gmx-1 py-4">
            <HiCheck className="mx-auto flex-shrink-0 w-5 h-5 text-neutral-900" />
          </td>
          <td className="gmx-1 py-4">
            <HiCheck className="mx-auto flex-shrink-0 w-5 h-5 text-neutral-900" />
          </td>
          <td className="gmx-1 py-4 justify-center">
            <HiCheck className="mx-auto flex-shrink-0 w-5 h-5 text-neutral-900" />
          </td>
        </tr>
        <tr className="border-b border-1 border-neutral-200">
          <td className="gmx-1 py-4">
            <span
              data-tooltip-target="email-otp-authentication-tooltip"
              data-tooltip-placement="top"
              className="underline hover:no-underline cursor-help"
            >
              Email (OTP) authentication
            </span>
            <div
              id="email-otp-authentication-tooltip"
              role="tooltip"
              className="text-wrap absolute z-10 invisible inline-block px-3 py-2 text-xs font-medium text-white transition-opacity duration-100 bg-neutral-900 rounded shadow-sm opacity-90 tooltip"
            >
              Authenticate users with a one-time code sent to their email
              <div className="tooltip-arrow" data-popper-arrow></div>
            </div>
          </td>
          <td className="gmx-1 py-4">
            <HiCheck className="mx-auto flex-shrink-0 w-5 h-5 text-neutral-900" />
          </td>
          <td className="gmx-1 py-4">
            <HiCheck className="mx-auto flex-shrink-0 w-5 h-5 text-neutral-900" />
          </td>
          <td className="gmx-1 py-4">
            <HiCheck className="mx-auto flex-shrink-0 w-5 h-5 text-neutral-900" />
          </td>
        </tr>
        <tr className="border-b border-1 border-neutral-200">
          <td className="gmx-1 py-4">
            <span
              data-tooltip-target="openid-connect-authentication-tooltip"
              data-tooltip-placement="top"
              className="underline hover:no-underline cursor-help"
            >
              OpenID Connect authentication
            </span>
            <div
              id="openid-connect-authentication-tooltip"
              role="tooltip"
              className="text-wrap absolute z-10 invisible inline-block px-3 py-2 text-xs font-medium text-white transition-opacity duration-100 bg-neutral-900 rounded shadow-sm opacity-90 tooltip"
            >
              Authenticate users with any OIDC-compatible provider
              <div className="tooltip-arrow" data-popper-arrow></div>
            </div>
          </td>
          <td className="gmx-1 py-4">
            <HiCheck className="mx-auto flex-shrink-0 w-5 h-5 text-neutral-900" />
          </td>
          <td className="gmx-1 py-4">
            <HiCheck className="mx-auto flex-shrink-0 w-5 h-5 text-neutral-900" />
          </td>
          <td className="gmx-1 py-4">
            <HiCheck className="mx-auto flex-shrink-0 w-5 h-5 text-neutral-900" />
          </td>
        </tr>
        <tr className="border-b border-1 border-neutral-200">
          <td className="gmx-1 py-4">
            <span
              data-tooltip-target="conditional-policies-tooltip"
              data-tooltip-placement="top"
              className="underline hover:no-underline cursor-help"
            >
              Conditional access policies
            </span>
            <div
              id="conditional-policies-tooltip"
              role="tooltip"
              className="text-wrap absolute z-10 invisible inline-block px-3 py-2 text-xs font-medium text-white transition-opacity duration-100 bg-neutral-900 rounded shadow-sm opacity-90 tooltip"
            >
              Allow access based on source IP, authentication method, time of
              day, or country.
              <div className="tooltip-arrow" data-popper-arrow></div>
            </div>
          </td>
          <td className="gmx-1 py-4 text-center">&#8212;</td>
          <td className="gmx-1 py-4">
            <HiCheck className="mx-auto flex-shrink-0 w-5 h-5 text-neutral-900" />
          </td>
          <td className="gmx-1 py-4">
            <HiCheck className="mx-auto flex-shrink-0 w-5 h-5 text-neutral-900" />
          </td>
        </tr>
        <tr className="border-b border-1 border-neutral-200">
          <td className="gmx-1 py-4">
            <span
              data-tooltip-target="custom-account-slug-tooltip"
              data-tooltip-placement="top"
              className="underline hover:no-underline cursor-help"
            >
              Custom account slug
            </span>
            <div
              id="custom-account-slug-tooltip"
              role="tooltip"
              className="text-wrap absolute z-10 invisible inline-block px-3 py-2 text-xs font-medium text-white transition-opacity duration-100 bg-neutral-900 rounded shadow-sm opacity-90 tooltip"
            >
              Customize the sign-in URL for your account. E.g.
              https://app.firezone.dev/your-organization
              <div className="tooltip-arrow" data-popper-arrow></div>
            </div>
          </td>
          <td className="gmx-1 py-4 text-center">&#8212;</td>
          <td className="gmx-1 py-4">
            <HiCheck className="mx-auto flex-shrink-0 w-5 h-5 text-neutral-900" />
          </td>
          <td className="gmx-1 py-4">
            <HiCheck className="mx-auto flex-shrink-0 w-5 h-5 text-neutral-900" />
          </td>
        </tr>
        <tr className="border-b border-1 border-neutral-200">
          <td className="gmx-1 py-4">
            <span
              data-tooltip-target="google-workspace-directory-sync-tooltip"
              data-tooltip-placement="top"
              className="underline hover:no-underline cursor-help"
            >
              Google Workspace directory sync
            </span>
            <div
              id="google-workspace-directory-sync-tooltip"
              role="tooltip"
              className="text-wrap absolute z-10 invisible inline-block px-3 py-2 text-xs font-medium text-white transition-opacity duration-100 bg-neutral-900 rounded shadow-sm opacity-90 tooltip"
            >
              Automatically sync users and groups from Google Workspace to
              Firezone
              <div className="tooltip-arrow" data-popper-arrow></div>
            </div>
          </td>
          <td className="gmx-1 py-4 text-center">&#8212;</td>
          <td className="gmx-1 py-4 text-center">&#8212;</td>
          <td className="gmx-1 py-4">
            <HiCheck className="mx-auto flex-shrink-0 w-5 h-5 text-neutral-900" />
          </td>
        </tr>
        <tr className="border-b border-1 border-neutral-200">
          <td className="gmx-1 py-4">
            <span
              data-tooltip-target="microsoft-entra-id-directory-sync-tooltip"
              data-tooltip-placement="top"
              className="underline hover:no-underline cursor-help"
            >
              Microsoft Entra ID directory sync
            </span>
            <div
              id="microsoft-entra-id-directory-sync-tooltip"
              role="tooltip"
              className="text-wrap absolute z-10 invisible inline-block px-3 py-2 text-xs font-medium text-white transition-opacity duration-100 bg-neutral-900 rounded shadow-sm opacity-90 tooltip"
            >
              Automatically sync users and groups from Microsoft Entra ID to
              Firezone
              <div className="tooltip-arrow" data-popper-arrow></div>
            </div>
          </td>
          <td className="gmx-1 py-4 text-center">&#8212;</td>
          <td className="gmx-1 py-4 text-center">&#8212;</td>
          <td className="gmx-1 py-4">
            <HiCheck className="mx-auto flex-shrink-0 w-5 h-5 text-neutral-900" />
          </td>
        </tr>
        <tr className="border-b border-1 border-neutral-200">
          <td className="gmx-1 py-4">
            <span
              data-tooltip-target="okta-directory-sync-tooltip"
              data-tooltip-placement="top"
              className="underline hover:no-underline cursor-help"
            >
              Okta directory sync
            </span>
            <div
              id="okta-directory-sync-tooltip"
              role="tooltip"
              className="text-wrap absolute z-10 invisible inline-block px-3 py-2 text-xs font-medium text-white transition-opacity duration-100 bg-neutral-900 rounded shadow-sm opacity-90 tooltip"
            >
              Automatically sync users and groups from Okta to Firezone
              <div className="tooltip-arrow" data-popper-arrow></div>
            </div>
          </td>
          <td className="gmx-1 py-4 text-center">&#8212;</td>
          <td className="gmx-1 py-4 text-center">&#8212;</td>
          <td className="gmx-1 py-4">
            <HiCheck className="mx-auto flex-shrink-0 w-5 h-5 text-neutral-900" />
          </td>
        </tr>
        <tr className="border-b border-1 border-neutral-200">
          <td className="gmx-1 py-4">
            <span
              data-tooltip-target="jumpcloud-directory-sync-tooltip"
              data-tooltip-placement="top"
              className="underline hover:no-underline cursor-help"
            >
              JumpCloud directory sync
            </span>
            <div
              id="jumpcloud-directory-sync-tooltip"
              role="tooltip"
              className="text-wrap absolute z-10 invisible inline-block px-3 py-2 text-xs font-medium text-white transition-opacity duration-100 bg-neutral-900 rounded shadow-sm opacity-90 tooltip"
            >
              Automatically sync users and groups from JumpCloud to Firezone
              <div className="tooltip-arrow" data-popper-arrow></div>
            </div>
          </td>
          <td className="gmx-1 py-4 text-center">&#8212;</td>
          <td className="gmx-1 py-4 text-center">&#8212;</td>
          <td className="gmx-1 py-4">
            <HiCheck className="mx-auto flex-shrink-0 w-5 h-5 text-neutral-900" />
          </td>
        </tr>
        <tr>
          <td
            colSpan={4}
            className="mx-1 pt-12 pb-4 text-lg uppercase font-semibold text-primary-450 tracking-light"
          >
            Security Features
          </td>
        </tr>
        <tr className="border-b border-1 border-neutral-200">
          <td className="gmx-1 py-4">
            <span
              data-tooltip-target="session-based-key-rotation-tooltip"
              data-tooltip-placement="top"
              className="underline hover:no-underline cursor-help"
            >
              Session-based key rotation
            </span>
            <div
              id="session-based-key-rotation-tooltip"
              role="tooltip"
              className="text-wrap absolute z-10 invisible inline-block px-3 py-2 text-xs font-medium text-white transition-opacity duration-100 bg-neutral-900 rounded shadow-sm opacity-90 tooltip"
            >
              Rotate WireGuard encryption keys each time a user signs in
              <div className="tooltip-arrow" data-popper-arrow></div>
            </div>
          </td>
          <td className="gmx-1 py-4">
            <HiCheck className="mx-auto flex-shrink-0 w-5 h-5 text-neutral-900" />
          </td>
          <td className="gmx-1 py-4">
            <HiCheck className="mx-auto flex-shrink-0 w-5 h-5 text-neutral-900" />
          </td>
          <td className="gmx-1 py-4">
            <HiCheck className="mx-auto flex-shrink-0 w-5 h-5 text-neutral-900" />
          </td>
        </tr>
        <tr className="border-b border-1 border-neutral-200">
          <td className="gmx-1 py-4">
            <span
              data-tooltip-target="geoip-mapping-tooltip"
              data-tooltip-placement="top"
              className="underline hover:no-underline cursor-help"
            >
              GeoIP Mapping
            </span>
            <div
              id="geoip-mapping-tooltip"
              role="tooltip"
              className="text-wrap absolute z-10 invisible inline-block px-3 py-2 text-xs font-medium text-white transition-opacity duration-100 bg-neutral-900 rounded shadow-sm opacity-90 tooltip"
            >
              Show where your users are connecting from
              <div className="tooltip-arrow" data-popper-arrow></div>
            </div>
          </td>
          <td className="gmx-1 py-4">
            <HiCheck className="mx-auto flex-shrink-0 w-5 h-5 text-neutral-900" />
          </td>
          <td className="gmx-1 py-4">
            <HiCheck className="mx-auto flex-shrink-0 w-5 h-5 text-neutral-900" />
          </td>
          <td className="gmx-1 py-4">
            <HiCheck className="mx-auto flex-shrink-0 w-5 h-5 text-neutral-900" />
          </td>
        </tr>
        <tr className="border-b border-1 border-neutral-200">
          <td className="gmx-1 py-4">
            <span
              data-tooltip-target="resource-access-logs-tooltip"
              data-tooltip-placement="top"
              className="underline hover:no-underline cursor-help"
            >
              Resource access logs
            </span>
            <div
              id="resource-access-logs-tooltip"
              role="tooltip"
              className="text-wrap absolute z-10 invisible inline-block px-3 py-2 text-xs font-medium text-white transition-opacity duration-100 bg-neutral-900 rounded shadow-sm opacity-90 tooltip"
            >
              See who accessed which Resource and when
              <div className="tooltip-arrow" data-popper-arrow></div>
            </div>
          </td>
          <td className="gmx-1 py-4 text-center">&#8212;</td>
          <td className="gmx-1 py-4">
            <HiCheck className="mx-auto flex-shrink-0 w-5 h-5 text-neutral-900" />
          </td>
          <td className="gmx-1 py-4">
            <HiCheck className="mx-auto flex-shrink-0 w-5 h-5 text-neutral-900" />
          </td>
        </tr>
        <tr className="border-b border-1 border-neutral-200">
          <td className="gmx-1 py-4">
            <span
              data-tooltip-target="traffic-restrictions-tooltip"
              data-tooltip-placement="top"
              className="underline hover:no-underline cursor-help"
            >
              Traffic restrictions
            </span>
            <div
              id="traffic-restrictions-tooltip"
              role="tooltip"
              className="text-wrap absolute z-10 invisible inline-block px-3 py-2 text-xs font-medium text-white transition-opacity duration-100 bg-neutral-900 rounded shadow-sm opacity-90 tooltip"
            >
              Restrict access to specific ports and protocols
              <div className="tooltip-arrow" data-popper-arrow></div>
            </div>
          </td>
          <td className="gmx-1 py-4 text-center">&#8212;</td>
          <td className="gmx-1 py-4">
            <HiCheck className="mx-auto flex-shrink-0 w-5 h-5 text-neutral-900" />
          </td>
          <td className="gmx-1 py-4">
            <HiCheck className="mx-auto flex-shrink-0 w-5 h-5 text-neutral-900" />
          </td>
        </tr>
        <tr className="border-b border-1 border-neutral-200">
          <td className="gmx-1 py-4">
            <span
              data-tooltip-target="compliance-audit-tooltip"
              data-tooltip-placement="top"
              className="underline hover:no-underline cursor-help"
            >
              Compliance certifications
            </span>
            <div
              id="compliance-audit-tooltip"
              role="tooltip"
              className="text-wrap absolute z-10 invisible inline-block px-3 py-2 text-xs font-medium text-white transition-opacity duration-100 bg-neutral-900 rounded shadow-sm opacity-90 tooltip"
            >
              Independent auditing for compliance with industry standards
              <div className="tooltip-arrow" data-popper-arrow></div>
            </div>
          </td>
          <td className="gmx-1 py-4 text-center">&#8212;</td>
          <td className="gmx-1 py-4 text-center">&#8212;</td>
          <td className="gmx-1 py-4 text-center">SOC 2</td>
        </tr>
        <tr className="border-b border-1 border-neutral-200">
          <td className="gmx-1 py-4">
            <span
              data-tooltip-target="pentesting-tooltip"
              data-tooltip-placement="top"
              className="underline hover:no-underline cursor-help"
            >
              Independent pentest
            </span>
            <div
              id="pentesting-tooltip"
              role="tooltip"
              className="text-wrap absolute z-10 invisible inline-block px-3 py-2 text-xs font-medium text-white transition-opacity duration-100 bg-neutral-900 rounded shadow-sm opacity-90 tooltip"
            >
              Penetration testing for security vulnerabilities conducted by a
              third party firm
              <div className="tooltip-arrow" data-popper-arrow></div>
            </div>
          </td>
          <td className="gmx-1 py-4 text-center">&#8212;</td>
          <td className="gmx-1 py-4 text-center">&#8212;</td>
          <td className="gmx-1 py-4 text-center">40 hours</td>
        </tr>
        <tr>
          <td
            colSpan={4}
            className="mx-1 pt-12 pb-4 text-lg uppercase font-semibold text-primary-450 tracking-light"
          >
            Support & Customer success
          </td>
        </tr>
        <tr className="border-b border-1 border-neutral-200">
          <td className="gmx-1 py-4">Community Forums</td>
          <td className="gmx-1 py-4">
            <HiCheck className="mx-auto flex-shrink-0 w-5 h-5 text-neutral-900" />
          </td>
          <td className="gmx-1 py-4">
            <HiCheck className="mx-auto flex-shrink-0 w-5 h-5 text-neutral-900" />
          </td>
          <td className="gmx-1 py-4">
            <HiCheck className="mx-auto flex-shrink-0 w-5 h-5 text-neutral-900" />
          </td>
        </tr>
        <tr className="border-b border-1 border-neutral-200">
          <td className="gmx-1 py-4">Community Discord</td>
          <td className="gmx-1 py-4">
            <HiCheck className="mx-auto flex-shrink-0 w-5 h-5 text-neutral-900" />
          </td>
          <td className="gmx-1 py-4">
            <HiCheck className="mx-auto flex-shrink-0 w-5 h-5 text-neutral-900" />
          </td>
          <td className="gmx-1 py-4">
            <HiCheck className="mx-auto flex-shrink-0 w-5 h-5 text-neutral-900" />
          </td>
        </tr>
        <tr className="border-b border-1 border-neutral-200">
          <td className="gmx-1 py-4">Priority Email</td>
          <td className="gmx-1 py-4 text-center">&#8212;</td>
          <td className="gmx-1 py-4">
            <HiCheck className="mx-auto flex-shrink-0 w-5 h-5 text-neutral-900" />
          </td>
          <td className="gmx-1 py-4">
            <HiCheck className="mx-auto flex-shrink-0 w-5 h-5 text-neutral-900" />
          </td>
        </tr>
        <tr className="border-b border-1 border-neutral-200">
          <td className="gmx-1 py-4">Dedicated Slack</td>
          <td className="gmx-1 py-4 text-center">&#8212;</td>
          <td className="gmx-1 py-4 text-center">&#8212;</td>
          <td className="gmx-1 py-4">
            <HiCheck className="mx-auto flex-shrink-0 w-5 h-5 text-neutral-900" />
          </td>
        </tr>
        <tr className="border-b border-1 border-neutral-200">
          <td className="gmx-1 py-4">
            <span
              data-tooltip-target="roadmap-acceleration-tooltip"
              data-tooltip-placement="top"
              className="underline hover:no-underline cursor-help"
            >
              Roadmap acceleration
            </span>
            <div
              id="roadmap-acceleration-tooltip"
              role="tooltip"
              className="text-wrap absolute z-10 invisible inline-block px-3 py-2 text-xs font-medium text-white transition-opacity duration-100 bg-neutral-900 rounded shadow-sm opacity-90 tooltip"
            >
              Shape the product roadmap with customized features and
              integrations
              <div className="tooltip-arrow" data-popper-arrow></div>
            </div>
          </td>
          <td className="gmx-1 py-4 text-center">&#8212;</td>
          <td className="gmx-1 py-4 text-center">&#8212;</td>
          <td className="gmx-1 py-4">
            <HiCheck className="mx-auto flex-shrink-0 w-5 h-5 text-neutral-900" />
          </td>
        </tr>
        <tr className="border-b border-1 border-neutral-200">
          <td className="gmx-1 py-4">
            <span
              data-tooltip-target="white-glove-onboarding-tooltip"
              data-tooltip-placement="top"
              className="underline hover:no-underline cursor-help"
            >
              White-glove onboarding
            </span>
            <div
              id="white-glove-onboarding-tooltip"
              role="tooltip"
              className="text-wrap absolute z-10 invisible inline-block px-3 py-2 text-xs font-medium text-white transition-opacity duration-100 bg-neutral-900 rounded shadow-sm opacity-90 tooltip"
            >
              Get personalized deployment support and training for your team
              <div className="tooltip-arrow" data-popper-arrow></div>
            </div>
          </td>
          <td className="gmx-1 py-4 text-center">&#8212;</td>
          <td className="gmx-1 py-4 text-center">&#8212;</td>
          <td className="gmx-1 py-4">
            <HiCheck className="mx-auto flex-shrink-0 w-5 h-5 text-neutral-900" />
          </td>
        </tr>
        <tr className="border-b border-1 border-neutral-200">
          <td className="gmx-1 py-4">
            <span
              data-tooltip-target="uptime-sla-tooltip"
              data-tooltip-placement="top"
              className="underline hover:no-underline cursor-help"
            >
              Uptime SLA
            </span>
            <div
              id="uptime-sla-tooltip"
              role="tooltip"
              className="text-wrap absolute z-10 invisible inline-block px-3 py-2 text-xs font-medium text-white transition-opacity duration-100 bg-neutral-900 rounded shadow-sm opacity-90 tooltip"
            >
              Guaranteed uptime for your Firezone service
              <div className="tooltip-arrow" data-popper-arrow></div>
            </div>
          </td>
          <td className="gmx-1 py-4 text-center">&#8212;</td>
          <td className="gmx-1 py-4 text-center">&#8212;</td>
          <td className="gmx-1 py-4 text-center">99.99%</td>
        </tr>
        <tr>
          <td
            colSpan={4}
            className="mx-1 pt-12 pb-4 text-lg uppercase font-semibold text-primary-450 tracking-light"
          >
            Billing & payment
          </td>
        </tr>
        <tr className="border-b border-1 border-neutral-200">
          <td className="gmx-1 py-4">
            <span
              data-tooltip-target="credit-card-tooltip"
              data-tooltip-placement="top"
              className="underline hover:no-underline cursor-help"
            >
              Payment by credit card
            </span>
            <div
              id="credit-card-tooltip"
              role="tooltip"
              className="text-wrap absolute z-10 invisible inline-block px-3 py-2 text-xs font-medium text-white transition-opacity duration-100 bg-neutral-900 rounded shadow-sm opacity-90 tooltip"
            >
              Pay for your subscription using a credit card
              <div className="tooltip-arrow" data-popper-arrow></div>
            </div>
          </td>
          <td className="gmx-1 py-4 text-center">&#8212;</td>
          <td className="gmx-1 py-4 text-center">
            <HiCheck className="mx-auto flex-shrink-0 w-5 h-5 text-neutral-900" />
          </td>
          <td className="gmx-1 py-4 text-center">
            <HiCheck className="mx-auto flex-shrink-0 w-5 h-5 text-neutral-900" />
          </td>
        </tr>
        <tr className="border-b border-1 border-neutral-200">
          <td className="gmx-1 py-4">
            <span
              data-tooltip-target="ach-tooltip"
              data-tooltip-placement="top"
              className="underline hover:no-underline cursor-help"
            >
              Payment by ACH transfer
            </span>
            <div
              id="ach-tooltip"
              role="tooltip"
              className="text-wrap absolute z-10 invisible inline-block px-3 py-2 text-xs font-medium text-white transition-opacity duration-100 bg-neutral-900 rounded shadow-sm opacity-90 tooltip"
            >
              Pay for your subscription using an ACH transfer
              <div className="tooltip-arrow" data-popper-arrow></div>
            </div>
          </td>
          <td className="gmx-1 py-4 text-center">&#8212;</td>
          <td className="gmx-1 py-4 text-center">
            <HiCheck className="mx-auto flex-shrink-0 w-5 h-5 text-neutral-900" />
          </td>
          <td className="gmx-1 py-4 text-center">
            <HiCheck className="mx-auto flex-shrink-0 w-5 h-5 text-neutral-900" />
          </td>
        </tr>
        <tr className="border-b border-1 border-neutral-200">
          <td className="gmx-1 py-4">
            <span
              data-tooltip-target="wire-transfer-tooltip"
              data-tooltip-placement="top"
              className="underline hover:no-underline cursor-help"
            >
              Payment by wire transfer
            </span>
            <div
              id="wire-transfer-tooltip"
              role="tooltip"
              className="text-wrap absolute z-10 invisible inline-block px-3 py-2 text-xs font-medium text-white transition-opacity duration-100 bg-neutral-900 rounded shadow-sm opacity-90 tooltip"
            >
              Pay your invoice using a wire transfer
              <div className="tooltip-arrow" data-popper-arrow></div>
            </div>
          </td>
          <td className="gmx-1 py-4 text-center">&#8212;</td>
          <td className="gmx-1 py-4 text-center">&#8212;</td>
          <td className="gmx-1 py-4 text-center">
            <HiCheck className="mx-auto flex-shrink-0 w-5 h-5 text-neutral-900" />
          </td>
        </tr>
        <tr className="border-b border-1 border-neutral-200">
          <td className="gmx-1 py-4">
            <span
              data-tooltip-target="annual-invoicing-tooltip"
              data-tooltip-placement="top"
              className="underline hover:no-underline cursor-help"
            >
              Annual invoicing
            </span>
            <div
              id="annual-invoicing-tooltip"
              role="tooltip"
              className="text-wrap absolute z-10 invisible inline-block px-3 py-2 text-xs font-medium text-white transition-opacity duration-100 bg-neutral-900 rounded shadow-sm opacity-90 tooltip"
            >
              Pay for your subscription annually
              <div className="tooltip-arrow" data-popper-arrow></div>
            </div>
          </td>
          <td className="gmx-1 py-4 text-center">&#8212;</td>
          <td className="gmx-1 py-4 text-center">&#8212;</td>
          <td className="gmx-1 py-4 text-center">
            <HiCheck className="mx-auto flex-shrink-0 w-5 h-5 text-neutral-900" />
          </td>
        </tr>
        <tr>
          <td></td>
          <td className="gmx-1 py-14 text-center">
            <Link href="https://app.firezone.dev/sign_up">
              <button
                type="button"
                className="md:text-lg md:py-2.5 text-sm sm:px-5 px-2.5 py-1.5 text-primary-450 font-semibold hover:font-bold tracking-tight rounded duration-0 hover:scale-105 transition transform shadow-lg border border-1 border-primary-450 hover:border-2"
              >
                Sign up
              </button>
            </Link>
          </td>
          <td className="gmx-1 py-14 text-center">
            <Link href="https://app.firezone.dev/sign_up">
              <button
                type="button"
                className="md:text-lg md:py-2.5 text-sm sm:px-5 px-2.5 py-1.5 text-primary-450 font-semibold hover:font-bold tracking-tight rounded duration-0 hover:scale-105 transition transform shadow-lg border border-1 border-primary-450 hover:border-2"
              >
                Sign up
              </button>
            </Link>
          </td>
          <td className="gmx-1 py-14 text-center">
            <Link href="/contact/sales">
              <button
                type="button"
                className="md:text-lg md:py-2.5 text-sm sm:px-5 px-2.5 py-1.5 text-white font-semibold hover:font-bold tracking-tight rounded duration-0 hover:scale-105 transition transform shadow-lg bg-primary-450"
              >
                Contact us
              </button>
            </Link>
          </td>
        </tr>
      </tbody>
    </table>
  );
}
