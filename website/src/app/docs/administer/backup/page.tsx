import Content from "./readme.mdx";
import { Metadata } from "next";

export const metadata: Metadata = {
  title: "Firezone Docs • Administer",
  description: "Firezone Documentation",
};

export default function Page() {
  return <Content />;
}
