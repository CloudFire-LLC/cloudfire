import { Metadata } from "next";
import Link from "next/link";

import "@/app/globals.css";
import "highlight.js/styles/a11y-dark.css";
import RootNavbar from "@/components/RootNavbar";
import Script from "next/script";
import Banner from "@/components/Banner";
import Providers from "@/components/Providers";
import Footer from "@/components/Footer";
import { Public_Sans } from "next/font/google";
const public_sans = Public_Sans({
  subsets: ["latin"],
  weight: ["100", "200", "300", "400", "500", "600", "700", "800", "900"],
});

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en">
      <Providers>
        <body className={public_sans.className}>
          <div className="min-h-screen h-auto antialiased">
            <RootNavbar />
            <Banner active>
              <p className="text-md font-medium text-center w-full text-neutral-200 ">
                <strong>Firezone 1.0 is coming!</strong> Rebuilt from the ground
                up with a cloud dashboard, native clients, and more.{" "}
                <Link
                  href="/blog/firezone-1-0"
                  className="underline text-accent-500  hover:no-underline"
                >
                  Read the blogpost
                </Link>{" "}
                or{" "}
                <Link
                  href="/product/early-access"
                  className="text-accent-500  underline hover:no-underline"
                >
                  request early access.
                </Link>
              </p>
            </Banner>
            {children}
            <Footer />
          </div>
          <Script
            id="hs-script-loader"
            async
            defer
            src="//js.hs-scripts.com/23723443.js"
          />
        </body>
      </Providers>
    </html>
  );
}
