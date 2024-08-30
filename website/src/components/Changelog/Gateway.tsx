import Entry from "./Entry";
import Entries from "./Entries";
import Link from "next/link";
import ChangeItem from "./ChangeItem";

export default function Gateway() {
  const href = "/dl/firezone-gateway/:version/:arch";
  const arches = ["x86_64", "aarch64", "armv7"];

  return (
    <Entries href={href} arches={arches} title="Gateway">
      {/*
      <Entry version="1.3.0" date={new Date(todo)}>
        <ul className="list-disc space-y-2 pl-4 mb-4">
          <ChangeItem pull="6434">
          Adds support for routing the Internet Resource for Clients.
          </ChangeItem>
        </ul>
      </Entry>
      */}
      <Entry version="1.2.0" date={new Date("2024-08-21")}>
        <ul className="list-disc space-y-2 pl-4 mb-4">
          <ChangeItem pull="5901">
            Implements glob-like matching of domains for DNS resources.
          </ChangeItem>
        </ul>
      </Entry>
      <Entry version="1.1.5" date={new Date("2024-08-13")}>
        <ul className="list-disc space-y-2 pl-4 mb-4">
          <ChangeItem pull="6276">
            Fixes a bug where relayed connections failed to establish after an
            idle period.
          </ChangeItem>
          <ChangeItem pull="6277">
            Fixes a bug where restrictive NATs caused connectivity problems.
          </ChangeItem>
        </ul>
      </Entry>
      <Entry version="1.1.4" date={new Date("2024-08-08")}>
        <ul className="list-disc space-y-2 pl-4 mb-4">
          <li className="pl-2">
            Removes `FIREZONE_ENABLE_MASQUERADE` env variable. Masquerading is
            now always enabled unconditionally.
          </li>
        </ul>
      </Entry>
      <Entry version="1.1.3" date={new Date("2024-08-02")}>
        <ul className="list-disc space-y-2 pl-4 mb-4">
          <li className="pl-2">
            Fixes{" "}
            <Link
              className="text-accent-500 underline hover:no-underline"
              href="https://github.com/firezone/firezone/pull/6117"
            >
              an issue
            </Link>{" "}
            where Gateways could become unresponsive after new versions of the
            Firezone infrastructure was deployed.
          </li>
        </ul>
      </Entry>
      <Entry version="1.1.2" date={new Date("2024-06-29")}>
        <ul className="list-disc space-y-2 pl-4 mb-4">
          <li className="pl-2">Reduces log noise for the default log level.</li>
        </ul>
      </Entry>
      <Entry version="1.1.1" date={new Date("2024-06-27")}>
        <ul className="list-disc space-y-2 pl-4 mb-4">
          <li className="pl-2">
            Fixes a minor connectivity issue that could occur for some DNS
            Resources.
          </li>
        </ul>
      </Entry>
      <Entry version="1.1.0" date={new Date("2024-06-19")}>
        <p className="mb-2 md:mb-4">
          This release introduces a new method of resolving and routing DNS
          Resources that is more reliable on some poorly-behaved networks. To
          use this new method, Client versions 1.1.0 or later are required.
          Client versions 1.0.x will continue to work with Gateway 1.1.x, but
          will not benefit from the new DNS resolution method.
        </p>
        <p>
          Read more about this change in the announcement post{" "}
          <Link
            href="/blog/improving-reliability-for-dns-resources"
            className="text-accent-500 underline hover:no-underline"
          >
            here
          </Link>
          .
        </p>
      </Entry>
      <Entry version="1.0.8" date={new Date("2024-06-17")}>
        This is a maintenance release with no major user-facing changes.
      </Entry>
      <Entry version="1.0.7" date={new Date("2024-06-12")}>
        This release fixes a bug where the incorrect Gateway version could be
        reported to the admin portal.
      </Entry>
      <Entry version="1.0.6" date={new Date("2024-06-11")}>
        This release contains connectivity fixes and performance improvements
        and is recommended for all users.
      </Entry>
      <Entry version="1.0.5" date={new Date("2024-05-22")}>
        Minor maintenance fixes.
      </Entry>
      <Entry version="1.0.4" date={new Date("2024-05-14")}>
        Fixes an issue detecting the correct architecture during installation
        and upgrades.
      </Entry>
      <Entry version="1.0.3" date={new Date("2024-05-08")}>
        Adds support for{" "}
        <Link
          href="/kb/deploy/resources#traffic-restrictions"
          className="hover:no-underline underline text-accent-500"
        >
          traffic restrictions
        </Link>
        .
      </Entry>
      <Entry version="1.0.2" date={new Date("2024-04-30")}>
        Fixes a big that caused invalid connections from being cleaned up
        properly.
      </Entry>
      <Entry version="1.0.1" date={new Date("224-04-29")}>
        Fixes a bug that could prevent the auto-upgrade script from working
        properly.
      </Entry>
      <Entry version="1.0.0" date={new Date("2024-04-24")}>
        Initial release.
      </Entry>
    </Entries>
  );
}
