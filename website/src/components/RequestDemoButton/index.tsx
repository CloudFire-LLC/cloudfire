"use client";
import Link from "next/link";

export default function RequestDemoButton() {
  return (
    <Link href="/contact/sales">
      <button
        type="button"
        className="text-white font-bold tracking-tight rounded duration-0 hover:scale-105 transition transform shadow-lg text-sm px-5 py-2.5 bg-accent-600 hover:bg-accent-700"
      >
        Request demo
      </button>
    </Link>
  );
}
