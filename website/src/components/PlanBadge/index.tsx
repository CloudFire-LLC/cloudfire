function planBadge(plan: string) {
  switch (plan.toLowerCase()) {
    case "enterprise":
      return (
        <span
          className="bg-primary-500 text-white text-xs font-semibold me-2 px-2.5 py-0.5 rounded"
          title="Feature available on the Enterprise plan"
        >
          ENTERPRISE
        </span>
      );
    case "team":
      return (
        <span
          className="bg-neutral-800 text-neutral-100 text-xs font-semibold me-2 px-2.5 py-0.5 rounded"
          title="Feature available on the Team plan"
        >
          TEAM
        </span>
      );
    case "starter":
      return (
        <span
          className="bg-neutral-200 text-neutral-900 text-xs font-semibold me-2 px-2.5 py-0.5 rounded"
          title="Feature available on the Starter plan"
        >
          STARTER
        </span>
      );
  }
}
export default function PlanBadge({
  children,
  plans,
}: {
  children: React.ReactNode;
  plans: Array<string>;
}) {
  const plansHtml = plans.map((plan) => planBadge(plan));
  return (
    <div className="flex flex-wrap justify-between">
      {children}
      <div>{plansHtml}</div>
    </div>
  );
}
