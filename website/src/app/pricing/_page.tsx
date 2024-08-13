"use client";

import { CustomerLogosColored } from "@/components/CustomerLogos";
import Toggle from "@/components/Toggle";
import { HiCheck } from "react-icons/hi2";
import Link from "next/link";
import PlanTable from "./plan_table";
import { useState } from "react";
import { Accordion } from "flowbite-react";

export default function _Page() {
  let [annual, setAnnual] = useState(true);
  let teamPrice: string;

  return (
    <>
      <section className="bg-white pb-14">
        <div className="flex justify-center mt-12">
          <span
            className={
              (annual ? "text-neutral-500 " : "text-neutral-900 ") +
              "font-medium me-3 text-lg uppercase"
            }
          >
            Monthly
          </span>
          <Toggle checked={annual} onChange={setAnnual} />
          <span
            className={
              (annual ? "text-neutral-900 " : "text-neutral-500 ") +
              "font-medium ms-3 text-lg uppercase"
            }
          >
            Annual
            <span className="text-sm text-neutral-700 text-primary-450">
              {" "}
              (Save 17%)
            </span>
          </span>
        </div>
        <div className="mx-auto max-w-screen-2xl md:grid md:grid-cols-3 pt-14 md:gap-4 px-4">
          <div className="p-8 bg-neutral-50 rounded border-2 border-neutral-200 mb-4">
            <h3 className="mb-4 text-2xl tracking-tight font-semibold text-primary-450">
              Starter
            </h3>
            <p className="mb-8">
              Secure remote access for individuals and small groups
            </p>
            <h2 className="mb-16 text-2xl sm:text-4xl tracking-tight font-semibold text-neutral-900">
              Free
            </h2>
            <div className="mb-24 w-full text-center">
              <Link href="https://app.firezone.dev/sign_up">
                <button
                  type="button"
                  className="bg-white w-64 text-lg px-5 py-2.5 md:w-44 md:text-sm md:px-3 md:py-2.5 lg:w-64 lg:text-lg lg:px-5 lg:py-2.5 border border-1 border-primary-450 hover:ring-1 hover:ring-primary-450 font-semibold tracking-tight rounded shadow-lg text-primary-450 duration-50 transition transform"
                >
                  Sign up
                </button>
              </Link>
            </div>
            <ul role="list" className="font-medium space-y-2">
              <li className="flex space-x-2.5">
                <HiCheck className="flex-shrink-0 w-5 h-5 text-neutral-900" />
                <span className="leading-tight text-neutral-900 ">
                  Up to 6 users
                </span>
              </li>
              <li className="flex space-x-2.5">
                <HiCheck className="flex-shrink-0 w-5 h-5 text-neutral-900" />
                <span className="leading-tight text-neutral-900 ">
                  Access your homelab or VPC from anywhere
                </span>
              </li>
              <li className="flex space-x-2.5">
                <HiCheck className="flex-shrink-0 w-5 h-5 text-neutral-900" />
                <span className="leading-tight text-neutral-900 ">
                  Native clients for Windows, Linux, macOS, iOS, Android
                </span>
              </li>
              <li className="flex space-x-2.5">
                <HiCheck className="flex-shrink-0 w-5 h-5 text-neutral-900" />
                <span className="leading-tight text-neutral-900 ">
                  Authenticate via email or OpenID Connect (OIDC)
                </span>
              </li>
              <li className="flex space-x-2.5">
                <HiCheck className="flex-shrink-0 w-5 h-5 text-neutral-900" />
                <span className="leading-tight text-neutral-900 ">
                  Load balancing and automatic failover
                </span>
              </li>
              <li className="flex space-x-2.5">
                <HiCheck className="flex-shrink-0 w-5 h-5 text-neutral-900" />
                <span className="leading-tight text-neutral-900 ">
                  No firewall configuration required
                </span>
              </li>
              <li className="flex space-x-2.5">
                <HiCheck className="flex-shrink-0 w-5 h-5 text-neutral-900" />
                <span className="leading-tight text-neutral-900 ">
                  Community Support
                </span>
              </li>
            </ul>
          </div>
          <div className="p-8 bg-neutral-50 rounded border-2 border-neutral-200 mb-4">
            <h3 className="mb-4 text-2xl tracking-tight font-semibold text-primary-450">
              Team
            </h3>
            <p className="mb-8">
              Zero trust network access for teams and organizations
            </p>
            <h2 className="mb-16 text-2xl sm:text-4xl tracking-tight font-semibold text-neutral-900">
              {annual && (
                <>
                  <span className="line-through">$5</span>
                  <span className="text-primary-450">$4.16</span>
                </>
              )}
              {!annual && <span>$5</span>}
              <span className="h-full">
                <span className="text-xs text-neutral-700 inline-block align-bottom ml-1 mb-1">
                  {" "}
                  per user / month
                </span>
              </span>
            </h2>
            <div className="mb-16 w-full text-center">
              <Link href="https://app.firezone.dev/sign_up">
                <button
                  type="button"
                  className="bg-white w-64 text-lg px-5 py-2.5 md:w-44 md:text-sm md:px-3 md:py-2.5 lg:w-64 lg:text-lg lg:px-5 lg:py-2.5 border border-1 border-primary-450 hover:ring-1 hover:ring-primary-450 font-semibold tracking-tight rounded shadow-lg text-primary-450 duration-50 transition transform"
                >
                  Sign up
                </button>
              </Link>
            </div>
            <p className="mb-2">
              <strong>Everything in Starter, plus:</strong>
            </p>
            <ul role="list" className="font-medium space-y-2">
              <li className="flex space-x-2.5">
                <HiCheck className="flex-shrink-0 w-5 h-5 text-neutral-900" />
                <span className="leading-tight text-neutral-900 ">
                  Up to 100 users
                </span>
              </li>
              <li className="flex space-x-2.5">
                <HiCheck className="flex-shrink-0 w-5 h-5 text-neutral-900" />
                <span className="leading-tight text-neutral-900 ">
                  Resource access logs
                </span>
              </li>
              <li className="flex space-x-2.5">
                <HiCheck className="flex-shrink-0 w-5 h-5 text-neutral-900" />
                <span className="leading-tight text-neutral-900 ">
                  Port and protocol traffic restrictions
                </span>
              </li>
              <li className="flex space-x-2.5">
                <HiCheck className="flex-shrink-0 w-5 h-5 text-neutral-900" />
                <span className="leading-tight text-neutral-900 ">
                  Conditional access policies
                </span>
              </li>
              <li className="flex space-x-2.5">
                <HiCheck className="flex-shrink-0 w-5 h-5 text-neutral-900" />
                <span className="leading-tight text-neutral-900 ">
                  Customize your account slug
                </span>
              </li>
              <li className="flex space-x-2.5">
                <HiCheck className="flex-shrink-0 w-5 h-5 text-neutral-900" />
                <span className="leading-tight text-neutral-900 ">
                  Faster relay network
                </span>
              </li>
              <li className="flex space-x-2.5">
                <HiCheck className="flex-shrink-0 w-5 h-5 text-neutral-900" />
                <span className="leading-tight text-neutral-900 ">
                  Priority email support
                </span>
              </li>
            </ul>
          </div>
          <div className="p-8 bg-neutral-950 text-neutral-50 rounded shadow border border-primary-450 mb-4">
            <div className="mb-4 flex items-center justify-between">
              <h3 className="text-2xl tracking-tight font-semibold text-primary-450">
                Enterprise
              </h3>
              <span className="font-semibold uppercase text-xs rounded bg-neutral-50 text-neutral-800 px-1 py-0.5">
                30-day trial
              </span>
            </div>
            <p className="mb-8 font-semibold">
              Compliance-ready security for large organizations
            </p>
            <h2 className="mb-16 text-2xl sm:text-4xl tracking-tight font-semibold">
              Contact us
            </h2>
            <div className="mb-16 w-full text-center">
              <Link href="/contact/sales">
                <button
                  type="button"
                  className="w-64 text-lg px-5 py-2.5 md:w-44 md:text-sm md:px-3 md:py-2.5 lg:w-64 lg:text-lg lg:px-5 lg:py-2.5 text-white font-semibold hover:ring-2 hover:ring-primary-450 tracking-tight transition transform duration-50 rounded bg-primary-450 shadow-lg shadow-primary-700"
                >
                  Request a demo
                </button>
              </Link>
            </div>
            <p className="mb-2">
              <strong>Everything in Team, plus:</strong>
            </p>
            <ul role="list" className="font-medium space-y-2">
              <li className="flex space-x-2.5">
                <HiCheck className="flex-shrink-0 w-5 h-5" />
                <span className="leading-tight">Unlimited users</span>
              </li>
              <li className="flex space-x-2.5">
                <HiCheck className="flex-shrink-0 w-5 h-5" />
                <span className="leading-tight">
                  Directory sync for Google, Entra ID, Okta, and JumpCloud
                </span>
              </li>
              <li className="flex space-x-2.5">
                <HiCheck className="flex-shrink-0 w-5 h-5" />
                <span className="leading-tight">
                  <span className="font-semibold text-primary-450">
                    Unthrottled
                  </span>{" "}
                  relay network
                </span>
              </li>
              <li className="flex space-x-2.5">
                <HiCheck className="flex-shrink-0 w-5 h-5" />
                <span className="leading-tight">
                  Dedicated Slack support channel
                </span>
              </li>
              <li className="flex space-x-2.5">
                <HiCheck className="flex-shrink-0 w-5 h-5" />
                <span className="leading-tight">Uptime SLAs</span>
              </li>
              <li className="flex space-x-2.5">
                <HiCheck className="flex-shrink-0 w-5 h-5" />
                <span className="leading-tight">
                  40-hour pentest &amp; SOC 2 reports
                </span>
              </li>
              <li className="flex space-x-2.5">
                <HiCheck className="flex-shrink-0 w-5 h-5" />
                <span className="leading-tight">Roadmap acceleration</span>
              </li>
              <li className="flex space-x-2.5">
                <HiCheck className="flex-shrink-0 w-5 h-5" />
                <span className="leading-tight">White-glove onboarding</span>
              </li>
              <li className="flex space-x-2.5">
                <HiCheck className="flex-shrink-0 w-5 h-5" />
                <span className="leading-tight">Annual invoicing</span>
              </li>
            </ul>
          </div>
        </div>
      </section>
      <section className="py-24 bg-gradient-to-b to-neutral-50 from-white">
        <CustomerLogosColored />
      </section>
      <section className="bg-neutral-50 py-14">
        <div className="mb-14 mx-auto max-w-screen-lg px-3">
          <h2 className="mb-14 justify-center text-4xl font-bold text-neutral-900">
            Compare plans
          </h2>
          <PlanTable />
        </div>
      </section>
      <section className="bg-neutral-100 border-t border-neutral-200 p-14">
        <div className="mx-auto max-w-screen-sm">
          <h2 className="mb-14 justify-center text-4xl font-bold text-neutral-900">
            FAQ
          </h2>

          <Accordion>
            <Accordion.Panel>
              <Accordion.Title>
                How long does it take to set up Firezone?
              </Accordion.Title>
              <Accordion.Content>
                A simple deployment takes{" "}
                <Link
                  href="/kb/quickstart"
                  className="hover:underline text-accent-500"
                >
                  less than 10 minutes{" "}
                </Link>
                and can be accomplished with by installing the{" "}
                <Link
                  href="/kb/client-apps"
                  className="hover:underline text-accent-500"
                >
                  Firezone Client
                </Link>{" "}
                and{" "}
                <Link
                  href="/kb/deploy/gateways"
                  className="hover:underline text-accent-500"
                >
                  deploying one or more Gateways
                </Link>
                .{" "}
                <Link href="/kb" className="hover:underline text-accent-500">
                  Visit our docs
                </Link>{" "}
                for more information and step by step instructions.
              </Accordion.Content>
            </Accordion.Panel>
            <Accordion.Panel>
              <Accordion.Title>Is there a self-hosted plan?</Accordion.Title>
              <Accordion.Content>
                All of the source code for the entire Firezone product is
                available at our{" "}
                <Link
                  href="https://www.github.com/firezone/firezone"
                  className="hover:underline text-accent-500"
                >
                  GitHub repository
                </Link>
                , and you're free to self-host Firezone for your organization
                without restriction. However, we don't offer documentation or
                support for self-hosting Firezone at this time.
              </Accordion.Content>
            </Accordion.Panel>
            <Accordion.Panel>
              <Accordion.Title>
                Do I need to rip and replace my current VPN to use Firezone?
              </Accordion.Title>
              <Accordion.Content>
                No. As long they're set up to access different resources, you
                can run Firezone alongside your existing remote access
                solutions, and switch over whenever you’re ready. There’s no
                need for any downtime or unnecessary disruptions.
              </Accordion.Content>
            </Accordion.Panel>
            <Accordion.Panel>
              <Accordion.Title>
                Can I try Firezone before I buy it?
              </Accordion.Title>
              <Accordion.Content>
                Yes. The Starter plan is free to use without limitation. No
                credit card is required to get started. The Enterprise plan
                includes a free pilot period to evaluate whether Firezone is a
                good fit for your organization.{" "}
                <Link
                  href="/contact/sales"
                  className="hover:underline text-accent-500"
                >
                  Contact sales
                </Link>{" "}
                to request a demo.
              </Accordion.Content>
            </Accordion.Panel>
            <Accordion.Panel>
              <Accordion.Title>
                My seat counts have changed. Can I adjust my plan?
              </Accordion.Title>
              <Accordion.Content>
                <p>Yes.</p>
                <p className="mt-2">
                  For the <strong>Team</strong> plan, you can add or remove
                  seats at any time. When adding seats, you'll be charged a
                  prorated amount for the remainder of the billing cycle. When
                  removing seats, the change will take effect at the end of the
                  billing cycle.
                </p>
                <p className="mt-2">
                  For the <strong>Enterprise</strong> plan, we will periodically
                  check your active seat count and will contact you to true up
                  your account once a quarter if there is a substantial change.
                  Enterprise plans{" "}
                  <strong>
                    will never become automatically locked or disabled due to an
                    increase in usage
                  </strong>
                  .
                </p>
              </Accordion.Content>
            </Accordion.Panel>
            <Accordion.Panel>
              <Accordion.Title>
                What happens to my data with Firezone enabled?
              </Accordion.Title>
              <Accordion.Content>
                Network traffic is always end-to-end encrypted, and by default,
                routes directly to Gateways running on your infrastructure. In
                rare circumstances, encrypted traffic can pass through our
                global relay network if a direct connection cannot be
                established. Firezone can never decrypt the contents of your
                traffic.
              </Accordion.Content>
            </Accordion.Panel>
            <Accordion.Panel>
              <Accordion.Title>
                How do I cancel or change my plan?
              </Accordion.Title>
              <Accordion.Content>
                For Starter and Team plans, you can downgrade by going to your
                Account settings in your Firezone admin portal. For Enterprise
                plans, contact your account manager for subscription updates. If
                you'd like to completely delete your account,{" "}
                <Link
                  href="mailto:support@firezone.dev"
                  className="hover:underline text-accent-500"
                >
                  contact support
                </Link>
                .
              </Accordion.Content>
            </Accordion.Panel>
            <Accordion.Panel>
              <Accordion.Title>When will I be billed?</Accordion.Title>
              <Accordion.Content>
                The Team plan is billed monthly on the same day you start
                service until canceled. Enterprise plans are billed annually.
              </Accordion.Content>
            </Accordion.Panel>
            <Accordion.Panel>
              <Accordion.Title>
                What payment methods are available?
              </Accordion.Title>
              <Accordion.Content>
                The Starter plan is free and does not require a credit card to
                get started. Team and Enterprise plans can be paid via credit
                card, ACH, or wire transfer.
              </Accordion.Content>
            </Accordion.Panel>
            <Accordion.Panel>
              <Accordion.Title>
                Do you offer special pricing for nonprofits and educational
                institutions?
              </Accordion.Title>
              <Accordion.Content>
                Yes. Not-for-profit organizations and educational institutions
                are eligible for a 50% discount.{" "}
                <Link
                  href="/contact/sales"
                  className="hover:underline text-accent-500"
                >
                  Contact sales
                </Link>{" "}
                to request the discount.
              </Accordion.Content>
            </Accordion.Panel>
            <Accordion.Panel>
              <Accordion.Title>
                What payment methods are available?
              </Accordion.Title>
              <Accordion.Content>
                The Starter plan is free and does not require a credit card to
                get started. Team and Enterprise plans can be paid via credit
                card, ACH, or wire transfer.
              </Accordion.Content>
            </Accordion.Panel>
          </Accordion>
        </div>
      </section>

      <section className="bg-neutral-100 border-t border-neutral-200 p-14">
        <div className="mx-auto max-w-screen-xl md:grid md:grid-cols-2">
          <div>
            <h2 className="w-full justify-center mb-8 text-2xl md:text-3xl font-semibold text-neutral-900">
              The WireGuard® solution for Enterprise.
            </h2>
          </div>
          <div className="mb-14 w-full text-center">
            <Link href="/contact/sales">
              <button
                type="button"
                className="w-64 text-white tracking-tight rounded duration-50 hover:ring-2 hover:ring-primary-300 transition transform shadow-lg text-lg px-5 py-2.5 bg-primary-450 font-semibold"
              >
                Request a demo
              </button>
            </Link>
          </div>
        </div>
      </section>
    </>
  );
}
