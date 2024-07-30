import Link from "next/link";
import Image from "next/image";
import ActionLink from "@/components/ActionLink";
import BattleCard from "@/components/BattleCard";
import { RunaCap } from "@/components/Badges";
import { Metadata } from "next";
import { CustomerLogosGrayscale } from "@/components/CustomerLogos";
import {
  HiShieldCheck,
  HiCheck,
  HiFingerPrint,
  HiArrowLongRight,
  HiGlobeAlt,
  HiHome,
  HiRocketLaunch,
} from "react-icons/hi2";
import {
  AppleIcon,
  WindowsIcon,
  LinuxIcon,
  AndroidIcon,
  ChromeIcon,
} from "@/components/Icons";

import {
  SlideIn,
  RotatingWords,
  Strike,
  FadeIn,
} from "@/components/Animations";
import SpeedChart from "@/components/Animations/SpeedChart";
import UpgradeDiagram from "@/components/Animations/UpgradeDiagram";
import ComplianceDiagram from "@/components/Animations/ComplianceDiagram";
import SimpleArchitecture from "@/components/Animations/SimpleArchitecture";
import { manrope } from "@/lib/fonts";
import "@/styles/hero.css";

export const metadata: Metadata = {
  title: "Firezone: Zero trust access that scales",
  description:
    "Firezone is a fast, flexible VPN replacement built on WireGuard® that eliminates tedious configuration and integrates with your identity provider.",
};

export default function Page() {
  return (
    <>
      <section className="bg-neutral-900 bg-hero pt-24 xl:pt-32 pb-14">
        <div className="flex flex-wrap mx-auto md:px-0 px-4 max-w-screen-md">
          <h1
            className={
              manrope.className +
              " mb-8 md:text-7xl text-4xl text-center shadow-inner font-medium tracking-tight leading-none text-neutral-300"
            }
          >
            Upgrade your VPN to zero-trust access
          </h1>
          <h3
            className={
              manrope.className +
              " md:mt-0 my-4 text-xl text-center text-neutral-300"
            }
          >
            Firezone is a fast, flexible VPN replacement built on WireGuard®
            that protects your workforce without tedious configuration.
          </h3>
          <div className="md:flex md:gap-x-12 mt-4 mx-auto">
            <div className="my-4 mr-4 flex items-center">
              <Link
                href="https://app.firezone.dev/sign_up"
                className="text-neutral-300 group inline-flex items-center py-0.5 text-lg font-semibold border-b-2 border-neutral-200 hover:border-primary-450 hover:text-primary-450 transition transform duration-50"
              >
                Get started for free
                <HiArrowLongRight className="group-hover:translate-x-1 group-hover:scale-110 duration-50 transition transform ml-2 -mr-1 w-7 h-7" />
              </Link>
            </div>
            <div className="mt-8 md:mt-0 flex items-center">
              <button
                type="button"
                className="group shadow-lg shadow-primary-700 text-lg w-48 inline-flex shadow-lg justify-center items-center py-3 px-5 font-semibold text-center text-white rounded bg-primary-450 hover:ring-1 hover:ring-primary-450 duration-50 transform transition"
              >
                <Link href="/contact/sales">Book a demo</Link>
                <HiArrowLongRight className="group-hover:translate-x-1 group-hover:scale-110 duration-50 transition transform ml-2 -mr-1 w-7 h-7" />
              </button>
            </div>
          </div>
        </div>
        <div className="mt-16 max-w-screen-lg mx-auto">
          <div className="text-sm mb-6 flex justify-center font-base text-neutral-600">
            Backed by{" "}
            <Image
              src="/images/yc-logo-gray.svg"
              alt="yc logo gray"
              width={100}
              height={40}
              className="mx-1.5"
            />{" "}
            and trusted by hundreds of organizations
          </div>
          <CustomerLogosGrayscale />
        </div>
      </section>

      {/* TODO: ACLs for the rest of us */}

      {/* Feature section 1: Secure access to your most sensitive resources in minutes. */}
      <section className="bg-white py-8 md:py-16">
        <div className="sm:mx-auto px-4 mb-4 md:mb-8 sm:text-center">
          <h3 className="text-2xl md:text-6xl tracking-tight font-bold inline-block">
            Supercharge your workforce in{" "}
            <span className="text-primary-450">minutes</span>.
          </h3>
        </div>

        <div className="mx-auto px-4 max-w-screen-md">
          <p className="text-md md:text-xl sm:text-center tracking-tight">
            Replace your obsolete VPN with a modern zero trust upgrade. Firezone
            supports the workflows and access patterns you're already familiar
            with, so you can get started in minutes and incrementally adopt more
            zero-trust patterns over time.
          </p>
        </div>

        <div className="flex justify-center items-center px-4 mx-auto mt-8 md:mt-16 max-w-screen-lg">
          <UpgradeDiagram />
        </div>

        <div className="flex items-stretch mx-auto mt-8 md:mt-16 gap-4 sm:gap-8 max-w-sm md:max-w-screen-lg grid md:grid-cols-3">
          <SlideIn
            direction="left"
            delay={0.5}
            duration={1}
            className="flex flex-col p-4"
          >
            <h4 className="mb-2 md:mb-4 text-md sm:text-lg md:text-xl font-semibold tracking-tight text-primary-450 uppercase">
              Flexible
            </h4>
            <p className="text-md sm:text-lg md:text-xl tracking-tight md:text-justify">
              Control access to VPCs, subnets, hosts by IP or DNS, and even
              public SaaS apps.
            </p>
          </SlideIn>
          <SlideIn
            direction="left"
            delay={0.75}
            duration={1}
            className="flex flex-col p-4 justify-center"
          >
            <h4 className="mb-2 md:mb-4 text-md sm:text-lg md:text-xl font-semibold tracking-tight text-primary-450 uppercase">
              Secure
            </h4>
            <p className="text-md sm:text-lg md:text-xl tracking-tight md:text-justify">
              Users and groups automatically sync with your identity provider,
              so access is revoked as soon as employees leave.
            </p>
          </SlideIn>
          <SlideIn
            direction="left"
            delay={1}
            duration={1}
            className="flex flex-col p-4 justify-center"
          >
            <h4 className="mb-2 md:mb-4 text-md sm:text-lg md:text-xl font-semibold tracking-tight text-primary-450 uppercase">
              Granular
            </h4>
            <p className="text-md sm:text-lg md:text-xl tracking-tight md:text-justify">
              Restrict access even further with port-level rules that allow
              access to some services but not others, even on the same host.
            </p>
          </SlideIn>
        </div>

        <div className="flex justify-center mt-8 md:mt-16">
          <ActionLink
            className="underline hover:no-underline text-md md:text-xl tracking-tight font-medium text-accent-500"
            href="/kb/deploy/resources"
          >
            Protect your resources
          </ActionLink>
        </div>
      </section>

      {/* Feature section 2: Achieve compliance in minutes, not weeks. */}
      <section className="bg-white py-8 md:py-16">
        <div className="sm:mx-auto px-4 mb-4 md:mb-8 sm:text-center">
          <h3 className="text-2xl md:text-6xl tracking-tight font-bold inline-block">
            Achieve compliance{" "}
            <span className="text-primary-450">without </span>
            the headache.
          </h3>
        </div>

        <div className="mx-auto px-4 max-w-screen-md">
          <p className="text-md md:text-xl sm:text-center tracking-tight">
            Connections are always end-to-end encrypted with keys that rotate
            daily, and are directly established between your Users and Gateways,
            so we can never see your data. Firezone's advanced Policy Engine
            logs who accessed what and when and can be configured to allow
            access only from certain countries, IPs, and timeframes, so you can
            easily demonstrate compliance with internal and external security
            audits.
          </p>
        </div>

        <div className="flex justify-center items-center px-4 md:px-0 mx-auto mt-8 md:mt-16 max-w-screen-lg">
          <ComplianceDiagram />
        </div>

        <div className="flex justify-center mt-8 md:mt-16">
          <ActionLink
            className="underline hover:no-underline text-md md:text-xl tracking-tight font-medium text-accent-500"
            href="/kb/architecture"
          >
            Read about Firezone's architecture
          </ActionLink>
        </div>
      </section>

      {/* Feature section 3: Add 2FA to WireGuard. */}
      <section className="bg-neutral-50 py-8 md:py-16">
        <div className="sm:mx-auto px-4 mb-4 md:mb-8 sm:text-center">
          <h3 className="text-2xl md:text-6xl tracking-tight font-bold inline-block">
            Add <span className="text-primary-450">two-factor </span>
            auth to WireGuard.
          </h3>
        </div>

        <div className="mx-auto px-4 max-w-screen-md">
          <p className="text-md md:text-xl sm:text-center tracking-tight">
            Looking for 2FA for WireGuard? Look no further. Firezone integrates
            with any OIDC-compatible identity provider to consistently enforce
            multi-factor authentication across your workforce.
          </p>
        </div>

        <div className="flex justify-center mt-8 md:mt-16">
          <SlideIn direction="top" delay={0.35} duration={1}>
            <Image
              width={1024}
              height={800}
              alt="Auth diagram"
              src="/images/auth.png"
              className="mx-auto px-4 md:px-0"
            />
          </SlideIn>
        </div>

        <div className="flex justify-center mt-8 md:mt-16">
          <ActionLink
            className="underline hover:no-underline text-md md:text-xl tracking-tight font-medium text-accent-500"
            href="/kb/authenticate"
          >
            Connect your identity provider
          </ActionLink>
        </div>
      </section>

      {/* Feature section 4: Say goodbye to bandwidth problems. */}
      <section className="bg-neutral-900 text-neutral-50 py-8 md:py-16">
        <div className="sm:mx-auto px-4 mb-4 md:mb-8 sm:text-center">
          <h3 className="text-2xl md:text-6xl tracking-tight font-bold inline-block">
            <Strike>Bandwidth problems.</Strike>
          </h3>
        </div>

        <div className="mx-auto mt-8 px-4 max-w-screen-md">
          <p className="text-md md:text-xl sm:text-center tracking-tight">
            Eliminate throughput bottlenecks that plague other VPNs. Firezone's
            load-balancing architecture scales horizontally to handle an
            unlimited number of connections to even the most bandwidth-intensive
            services. Need more speed? Just add more Gateways.
          </p>
        </div>

        <div className="flex justify-center max-w-screen-sm mx-auto mt-12 px-4 md:px-0">
          <SpeedChart />
        </div>

        <div className="flex justify-center mt-4 md:mt-16">
          <ActionLink
            className="underline hover:no-underline text-md md:text-xl tracking-tight font-semibold text-primary-450"
            href="/kb/use-cases/scale-vpc-access"
          >
            Scale access to your VPCs
          </ActionLink>
        </div>
      </section>

      {/* Feature section 5: No more open firewall ports. */}
      <section className="bg-white py-8 md:py-16">
        <div className="sm:mx-auto px-4 mb-4 md:mb-8 sm:text-center">
          <h3 className="text-2xl md:text-6xl tracking-tight font-bold inline-block">
            Say <span className="text-primary-450">goodbye</span> to firewall
            configuration.
          </h3>
        </div>

        <div className="mx-auto px-4 max-w-screen-md">
          <p className="text-md md:text-xl sm:text-center tracking-tight">
            Firezone securely punches through firewalls with ease, so keep those
            ports closed. Connections pick the shortest path and your attack
            surface is minimized, keeping your most sensitive resources
            invisible to attackers.
          </p>
        </div>

        <div className="flex justify-center items-center px-4 md:px-0 mx-auto mt-8 md:mt-16 max-w-screen-lg">
          <SimpleArchitecture />
        </div>

        <div className="flex justify-center mt-8">
          <ActionLink
            className="underline hover:no-underline text-md md:text-xl tracking-tight font-medium text-accent-500"
            href="/kb/deploy"
          >
            Make your resources invisible
          </ActionLink>
        </div>
      </section>

      {/* Feature section 6: Runs everywhere your business does. */}
      <section className="bg-neutral-50 py-8 md:py-16">
        <div className="sm:mx-auto px-4 mb-4 md:mb-8 sm:text-center">
          <h3 className="text-2xl md:text-6xl tracking-tight font-bold inline-block">
            Runs <span className="text-primary-450">everywhere </span>
            your business does.
          </h3>
        </div>

        <div className="mx-auto px-4 mt-8 max-w-screen-lg grid sm:grid-cols-2 gap-8 lg:gap-16">
          <div className="flex flex-col p-4">
            <div className="mb-12 grid grid-cols-2 gap-4">
              <div className="p-4 flex items-center justify-center bg-white rounded-lg border border-2 border-neutral-200">
                <AppleIcon size={12} href="/kb/client-apps/macos-client">
                  <span className="inline-block pt-4 w-full text-center">
                    macOS
                  </span>
                </AppleIcon>
              </div>
              <div className="p-4 flex items-center justify-center bg-white rounded-lg border border-2 border-neutral-200">
                <WindowsIcon size={12} href="/kb/client-apps/windows-client">
                  <span className="inline-block pt-4 w-full text-center">
                    Windows
                  </span>
                </WindowsIcon>
              </div>
              <div className="p-4 flex items-center justify-center bg-white rounded-lg border border-2 border-neutral-200">
                <LinuxIcon size={12} href="/kb/client-apps/linux-client">
                  <span className="inline-block pt-4 w-full text-center">
                    Linux
                  </span>
                </LinuxIcon>
              </div>
              <div className="p-4 flex items-center justify-center bg-white rounded-lg border border-2 border-neutral-200">
                <AndroidIcon size={12} href="/kb/client-apps/android-client">
                  <span className="inline-block pt-4 w-full text-center">
                    Android
                  </span>
                </AndroidIcon>
              </div>
              <div className="p-4 flex items-center justify-center bg-white rounded-lg border border-2 border-neutral-200">
                <ChromeIcon size={12} href="/kb/client-apps/android-client">
                  <span className="inline-block pt-4 w-full text-center">
                    ChromeOS
                  </span>
                </ChromeIcon>
              </div>
              <div className="p-4 flex items-center justify-center bg-white rounded-lg border border-2 border-neutral-200">
                <AppleIcon size={12} href="/kb/client-apps/ios-client">
                  <span className="inline-block pt-4 w-full text-center">
                    iOS
                  </span>
                </AppleIcon>
              </div>
            </div>
            <div className="mt-auto">
              <p className="text-md md:text-xl tracking-tight md:text-justify">
                Clients are available for every major platform, require no
                configuration, and stay connected even when switching WiFi
                networks.
              </p>
              <p className="mt-4">
                <ActionLink
                  className="underline hover:no-underline text-md md:text-xl tracking-tight font-medium text-accent-500"
                  href="/kb/client-apps"
                >
                  Download Client apps
                </ActionLink>
              </p>
            </div>
          </div>
          <div className="flex flex-col p-4">
            <div className="mb-12">
              <div className="py-0.5 flex flex-col justify-between space-y-8 md:space-y-12">
                <div className="mx-8 md:mx-16 flex justify-start">
                  <Image
                    width={200}
                    height={200}
                    alt="Gateway"
                    src="/images/docker.svg"
                  />
                </div>
                <div className="mx-8 md:mx-16 flex justify-end">
                  <Image
                    width={200}
                    height={200}
                    alt="Gateway"
                    src="/images/terraform.svg"
                  />
                </div>
                <div className="mx-8 md:mx-16 flex justify-start">
                  <Image
                    width={200}
                    height={200}
                    alt="Gateway"
                    src="/images/kubernetes.svg"
                  />
                </div>
                <div className="mx-8 md:mx-16 flex justify-end">
                  <Image
                    width={200}
                    height={200}
                    alt="Gateway"
                    src="/images/pulumi.svg"
                  />
                </div>
              </div>
              <pre className="mt-4 md:mt-8 text-xs p-2 bg-neutral-900 rounded shadow text-neutral-50 text-wrap">
                <code>
                  <strong>FIREZONE_TOKEN</strong>=&lt;your-token&gt; \<br /> ./
                  <strong>firezone-gateway</strong>
                </code>
              </pre>
            </div>
            <div className="mt-auto">
              <p className="text-md md:text-xl tracking-tight md:text-justify">
                Gateways are lightweight Linux binaries you deploy anywhere you
                need access. Just configure a token with your preferred
                orchestration tool and you're done.
              </p>
              <p className="mt-4">
                <ActionLink
                  className="underline hover:no-underline text-md md:text-xl tracking-tight font-medium text-accent-500"
                  href="/kb/deploy/gateways"
                >
                  Deploy your first Gateway
                </ActionLink>
              </p>
            </div>
          </div>
        </div>
      </section>

      {/* Feature section 7: Open source for transparency and trust. */}
      <section className="bg-white py-8 md:py-16">
        <div className="sm:mx-auto px-4 mb-4 md:mb-8 sm:text-center">
          <h3 className="text-xl sm:text-2xl md:text-6xl tracking-tight font-bold inline-block">
            <span className="text-primary-450">Open source</span> for
            transparency and trust.
          </h3>
        </div>

        <div className="mx-auto px-4 max-w-screen-md">
          <p className="text-md md:text-xl sm:text-center tracking-tight">
            How can you trust a zero-trust solution if you can't see its source?
            We build Firezone in the open so anyone can make sure it does
            exactly what we claim it does, and nothing more.
          </p>
        </div>

        <div className="mx-auto flex max-w-screen-md justify-center mt-8">
          <Image
            src="https://api.star-history.com/svg?repos=firezone/firezone&type=Date"
            alt="Firezone stars"
            width={800}
            height={600}
            className="mx-auto px-4 md:px-0"
          />
        </div>
        <div className="flex flex-col justify-center items-center px-4">
          <div className="w-full flex flex-wrap max-w-screen-sm justify-between mt-8">
            <div className="mx-auto w-64 mb-8 inline-flex justify-center">
              <RunaCap />
            </div>
            <div className="mx-auto w-64 mb-8 inline-flex justify-center">
              <ActionLink
                className="flex underline hover:no-underline text-md md:text-xl tracking-tight font-medium text-accent-500"
                href="https://www.github.com/firezone/firezone"
              >
                Leave us a star
              </ActionLink>
            </div>
          </div>
        </div>
      </section>

      {/* Use cases */}
      <section className="border-t border-neutral-200 py-8 md:py-16 bg-neutral-100">
        <div className="mx-auto max-w-screen-lg">
          <div className="px-4 flex flex-wrap">
            <h3 className="mb-2 text-2xl md:text-4xl tracking-tight font-bold mr-1">
              Yes, you can use Firezone to{" "}
            </h3>
            <h3 className="mb-2 text-2xl md:text-4xl tracking-tight font-bold">
              <RotatingWords
                className="underline text-primary-450 mx-0.5 sm:mx-1 inline-flex"
                words={[
                  "secure DNS for your workforce",
                  "securely access GitLab",
                  "scale access to your VPC",
                  "access your homelab",
                  "route through a public IP",
                  "access your Postgres DB",
                  "tunnel IPv6 over IPv4",
                  "restrict access to GitHub",
                  "tunnel to a remote host",
                ]}
              />
            </h3>
          </div>
          <div className="px-4 flex flex-wrap mt-8">
            <h3 className="mb-2 text-xl md:text-2xl tracking-tight font-semibold">
              Here are just a few ways customers are using Firezone:
            </h3>
          </div>
          <div className="gap-4 items-center pt-8 px-4 mx-auto md:grid md:grid-cols-2 xl:gap-8 sm:pt-12 lg:px-6">
            <SlideIn delay={0.5} direction="right">
              <div className="bg-neutral-50 p-8 mt-4 md:mt-0 border border-neutral-200">
                <div className="flex items-center space-x-2.5">
                  <HiShieldCheck className=" lex-shrink-0 w-6 h-6 text-accent-600" />
                  <h4 className="text-xl tracking-tight font-bold text-neutral-900">
                    VPN Replacement
                  </h4>
                </div>
                <p className="mt-8 text-neutral-900 text-xl">
                  Remote employees can securely access office networks, cloud
                  VPCs, and other private subnets and resources from anywhere in
                  the world, on any device.
                </p>
                <ul role="list" className="my-6 lg:mb-0 space-y-4">
                  <li className="flex space-x-2.5">
                    <HiCheck className="flex-shrink-0 w-5 h-5 text-neutral-900" />
                    <span className="leading-tight text-lg text-neutral-900 ">
                      Easy to use, no training required
                    </span>
                  </li>
                  <li className="flex space-x-2.5">
                    <HiCheck className="flex-shrink-0 w-5 h-5 text-neutral-900" />
                    <span className="leading-tight text-lg text-neutral-900 ">
                      Authenticate with virtually any IdP
                    </span>
                  </li>
                  <li className="flex space-x-2.5">
                    <HiCheck className="flex-shrink-0 w-5 h-5 text-neutral-900" />
                    <span className="leading-tight text-lg text-neutral-900 ">
                      Highly available Gateways
                    </span>
                  </li>
                  <li className="flex space-x-2.5">
                    <HiCheck className="flex-shrink-0 w-5 h-5 text-neutral-900" />
                    <span className="leading-tight text-lg text-neutral-900 ">
                      Modern encryption and authentication
                    </span>
                  </li>
                </ul>
              </div>
            </SlideIn>
            <SlideIn delay={0.5} direction="left">
              <div className="bg-neutral-50 p-8 mt-4 md:mt-0 border border-neutral-200">
                <div className="flex items-center space-x-2.5">
                  <HiRocketLaunch className="flex-shrink-0 w-6 h-6 text-accent-600" />
                  <h4 className="text-xl tracking-tight font-bold text-neutral-900 ">
                    Infrastructure Access
                  </h4>
                </div>
                <p className="mt-8 text-neutral-900 text-xl">
                  Empower engineers and DevOps to manage their team’s access to
                  technical resources like test/prod servers both on-prem and in
                  the cloud.
                </p>
                <ul role="list" className="my-6 lg:mb-0 space-y-4">
                  <li className="flex space-x-2.5">
                    <HiCheck className="flex-shrink-0 w-5 h-5 text-neutral-900" />
                    <span className="leading-tight text-lg text-neutral-900 ">
                      Service accounts and headless clients
                    </span>
                  </li>
                  <li className="flex space-x-2.5">
                    <HiCheck className="flex-shrink-0 w-5 h-5 text-neutral-900" />
                    <span className="leading-tight text-lg text-neutral-900 ">
                      Multiple admins per account
                    </span>
                  </li>
                  <li className="flex space-x-2.5">
                    <HiCheck className="flex-shrink-0 w-5 h-5 text-neutral-900" />
                    <span className="leading-tight text-lg text-neutral-900 ">
                      Docker and Terraform integrations
                    </span>
                  </li>
                  <li className="flex space-x-2.5">
                    <HiCheck className="flex-shrink-0 w-5 h-5 text-neutral-900" />
                    <span className="leading-tight text-lg text-neutral-900 ">
                      Automatically sync users and groups from your IdP
                    </span>
                  </li>
                </ul>
              </div>
            </SlideIn>
            <SlideIn delay={0.5} direction="right">
              <div className="bg-neutral-50 p-8 mt-4 md:mt-0 border border-neutral-200">
                <div className="flex items-center space-x-2.5">
                  <HiGlobeAlt className=" lex-shrink-0 w-6 h-6 text-accent-600" />
                  <h4 className="text-xl tracking-tight font-bold text-neutral-900 ">
                    Internet Security
                  </h4>
                </div>
                <p className="mt-8 text-neutral-900 text-xl">
                  Route sensitive internet traffic through a trusted gateway to
                  keep remote employees more secure, even when they’re traveling
                  or using public WiFi.
                </p>
                <ul role="list" className="my-6 lg:mb-0 space-y-4">
                  <li className="flex space-x-2.5">
                    <HiCheck className="flex-shrink-0 w-5 h-5 text-neutral-900" />
                    <span className="leading-tight text-lg text-neutral-900 ">
                      Native clients for all major platforms
                    </span>
                  </li>
                  <li className="flex space-x-2.5">
                    <HiCheck className="flex-shrink-0 w-5 h-5 text-neutral-900" />
                    <span className="leading-tight text-lg text-neutral-900 ">
                      Enforce MFA / 2FA
                    </span>
                  </li>
                  <li className="flex space-x-2.5">
                    <HiCheck className="flex-shrink-0 w-5 h-5 text-neutral-900" />
                    <span className="leading-tight text-lg text-neutral-900 ">
                      Filter malicious or unwanted DNS requests
                    </span>
                  </li>
                  <li className="flex space-x-2.5">
                    <HiCheck className="flex-shrink-0 w-5 h-5 text-neutral-900" />
                    <span className="leading-tight text-lg text-neutral-900 ">
                      Monitor and audit each attempted connection
                    </span>
                  </li>
                </ul>
              </div>
            </SlideIn>
            <SlideIn delay={0.5} direction="left">
              <div className="bg-neutral-50 p-8 mt-4 md:mt-0 border border-neutral-200">
                <div className="flex items-center space-x-2.5">
                  <HiHome className="flex-shrink-0 w-6 h-6 text-accent-600" />
                  <h4 className="text-xl tracking-tight font-bold text-neutral-900 ">
                    Homelab Access
                  </h4>
                </div>
                <p className="mt-8 text-neutral-900 text-xl">
                  Securely access your home network, and services like Plex,
                  security cameras, a Raspberry Pi, and other self-hosted apps
                  when you’re away from home.
                </p>
                <ul role="list" className="my-6 lg:mb-0 space-y-4">
                  <li className="flex space-x-2.5">
                    <HiCheck className="flex-shrink-0 w-5 h-5 text-neutral-900" />
                    <span className="leading-tight text-lg text-neutral-900 ">
                      Easy to setup and simple to manage
                    </span>
                  </li>
                  <li className="flex space-x-2.5">
                    <HiCheck className="flex-shrink-0 w-5 h-5 text-neutral-900" />
                    <span className="leading-tight text-lg text-neutral-900 ">
                      Authenticate with Email OTP or OIDC
                    </span>
                  </li>
                  <li className="flex space-x-2.5">
                    <HiCheck className="flex-shrink-0 w-5 h-5 text-neutral-900" />
                    <span className="leading-tight text-lg text-neutral-900 ">
                      Reliable NAT traversal
                    </span>
                  </li>
                  <li className="flex space-x-2.5">
                    <HiCheck className="flex-shrink-0 w-5 h-5 text-neutral-900" />
                    <span className="leading-tight text-lg text-neutral-900 ">
                      Invite friends and family to your private network
                    </span>
                  </li>
                </ul>
              </div>
            </SlideIn>
          </div>
          <div className="flex justify-center mt-8 md:mt-16">
            <ActionLink
              className="underline hover:no-underline text-md md:text-xl tracking-tight font-medium text-accent-500"
              href="/kb/use-cases"
            >
              See more use cases
            </ActionLink>
          </div>
        </div>
      </section>

      <section className="border-t border-neutral-200 py-24 bg-white">
        <BattleCard />
      </section>
    </>
  );
}
