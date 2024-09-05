import Link from "next/link";
import { Route } from "next";

import Image from "next/image";

export default function SummaryCard({
  children,
  date,
  href,
  title,
  authorName,
  authorAvatarSrc,
  type,
  src,
}: {
  children: React.ReactNode;
  date: string;
  href: URL | Route<string>;
  title: string;
  authorName: string;
  authorAvatarSrc: string;
  type: string;
  src?: string;
}) {
  return (
    <article className={`relative flex flex-col-reverse md:flex-row py-2 md:py-6 px-2 md:px-4`}>
      <Link href={href}>
        <div className="inset-0 mx-4 absolute hover:bg-neutral-100 bg-white mt-2 rounded-2xl cursor-pointer" />
      </Link>
      <div className="w-full md:w-2/3 relative pointer-events-none z-10 py-2 md:py-0 px-8 md:px-0 mx-0 md:mx-8">
        <div className="flex justify-between items-center mb-2">
          <span className="uppercase text-primary-450 font-semibold text-sm inline-flex items-center">
            {type}
          </span>
        </div>
        <h2 className="mb-3 text-2xl font-bold tracking-tight text-neutral-800 ">
          {title}
        </h2>
        <div className="summary-inner mb-6 font-regular text-neutral-800 z-10">
          {children}
        </div>
        <div className="flex gap-3 items-center text-sm">
          <div className="flex items-center space-x-3">
            <Image
              width={28}
              height={28}
              className="w-7 h-7 rounded-full"
              src={authorAvatarSrc}
              alt={authorName + " avatar"}
            />
            <span className="font-semibold">{authorName}</span>
          </div>
          <span className="font-semibold text-neutral-600">{date}</span>
        </div>
      </div>
      {src && (
        <div className="relative z-10 w-full md:w-1/3 place-content-center p-8 md:p-0 mx-0 md:mx-8 pointer-events-none">
          <Image
            src={src}
            width={800}
            height={800}
            alt="Article Image"
            className="object-cover rounded-lg object-center"
          />
        </div>
      )}
    </article>
  );
}
