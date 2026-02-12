"use client";

import Link from "next/link";
import { Check, ArrowRight, ChevronDown } from "lucide-react";
import { useState } from "react";

const tiers = [
  {
    name: "Free",
    price: "$0",
    period: "forever",
    description: "Perfect for personal projects and development.",
    cta: "Get Started",
    ctaStyle: "border border-border bg-surface hover:bg-surface-2 text-foreground",
    features: [
      "1 tunnel",
      "Random subdomains",
      "HTTP & HTTPS tunnels",
      "Request inspector",
      "Community support",
      "Self-host anywhere",
    ],
  },
  {
    name: "Pro",
    price: "$10",
    period: "/month",
    description: "For professionals who need custom domains and more tunnels.",
    cta: "Start Free Trial",
    ctaStyle: "bg-accent text-black hover:bg-accent/90",
    popular: true,
    features: [
      "Unlimited tunnels",
      "Custom domains",
      "TCP & TLS tunnels",
      "Webhook replay",
      "Team members (up to 5)",
      "Priority support",
      "API access",
      "99.9% uptime SLA",
    ],
  },
  {
    name: "Enterprise",
    price: "Custom",
    period: "",
    description: "For organizations with advanced security and compliance needs.",
    cta: "Contact Sales",
    ctaStyle: "border border-border bg-surface hover:bg-surface-2 text-foreground",
    features: [
      "Everything in Pro",
      "Unlimited team members",
      "SSO / SAML",
      "Audit logs",
      "Dedicated support",
      "Custom deployment",
      "SLA customization",
      "On-premise option",
    ],
  },
];

const faqs = [
  {
    q: "Can I self-host SubTunnel for free?",
    a: "Yes! SubTunnel is open source under the MIT license. You can self-host the server on your own infrastructure at no cost. The paid plans are for our managed service and additional features.",
  },
  {
    q: "How does SubTunnel compare to ngrok?",
    a: "SubTunnel is a self-hosted alternative to ngrok. The key differences are: you own your data, there's no vendor lock-in, and the core is open source. For teams that need data sovereignty or want to run tunnels on their own infrastructure, SubTunnel is the better choice.",
  },
  {
    q: "What happens if I exceed my plan limits?",
    a: "We'll notify you when you're approaching limits. We never cut off your tunnels unexpectedly. You can upgrade at any time, and downgrades take effect at the next billing cycle.",
  },
  {
    q: "Do you offer a free trial for Pro?",
    a: "Yes, Pro comes with a 14-day free trial. No credit card required to start. You'll get full access to all Pro features during the trial.",
  },
  {
    q: "Can I use SubTunnel for production traffic?",
    a: "Absolutely. SubTunnel is designed for production use with built-in TLS, health checks, and high availability. Many teams use it for webhook development, API testing, and exposing internal services.",
  },
];

function FAQ({
  q,
  a,
}: {
  q: string;
  a: string;
}) {
  const [open, setOpen] = useState(false);
  return (
    <div className="border-b border-border last:border-0">
      <button
        onClick={() => setOpen(!open)}
        className="flex w-full items-center justify-between py-5 text-left"
      >
        <span className="text-sm font-medium pr-4">{q}</span>
        <ChevronDown
          className={`w-4 h-4 text-muted shrink-0 transition-transform ${
            open ? "rotate-180" : ""
          }`}
        />
      </button>
      {open && (
        <p className="pb-5 text-sm text-muted leading-relaxed">{a}</p>
      )}
    </div>
  );
}

export default function PricingPage() {
  return (
    <div>
      <section className="relative py-24 md:py-32">
        <div className="absolute inset-0 bg-[radial-gradient(ellipse_at_top,rgba(0,212,255,0.06),transparent_60%)]" />
        <div className="relative mx-auto max-w-6xl px-6">
          <div className="text-center mb-16">
            <h1 className="text-4xl md:text-5xl font-bold tracking-tight">
              Simple, transparent pricing
            </h1>
            <p className="mt-4 text-lg text-muted max-w-lg mx-auto">
              Start free, scale as you grow. Self-host for free, or let us manage it.
            </p>
          </div>

          <div className="grid md:grid-cols-3 gap-6 max-w-5xl mx-auto">
            {tiers.map((tier) => (
              <div
                key={tier.name}
                className={`relative rounded-xl border p-8 flex flex-col ${
                  tier.popular
                    ? "border-accent/50 bg-surface glow-accent"
                    : "border-border bg-surface/50"
                }`}
              >
                {tier.popular && (
                  <div className="absolute -top-3 left-1/2 -translate-x-1/2">
                    <span className="inline-flex items-center rounded-full bg-accent px-3 py-1 text-xs font-medium text-black">
                      Most Popular
                    </span>
                  </div>
                )}
                <div>
                  <h3 className="text-lg font-semibold">{tier.name}</h3>
                  <div className="mt-4 flex items-baseline gap-1">
                    <span className="text-4xl font-bold">{tier.price}</span>
                    <span className="text-sm text-muted">{tier.period}</span>
                  </div>
                  <p className="mt-3 text-sm text-muted">{tier.description}</p>
                </div>
                <div className="mt-8 flex-1">
                  <ul className="space-y-3">
                    {tier.features.map((f) => (
                      <li key={f} className="flex items-start gap-3">
                        <Check className="w-4 h-4 text-accent mt-0.5 shrink-0" />
                        <span className="text-sm text-muted">{f}</span>
                      </li>
                    ))}
                  </ul>
                </div>
                <Link
                  href="/docs"
                  className={`mt-8 inline-flex h-10 items-center justify-center rounded-lg text-sm font-medium transition-colors ${tier.ctaStyle}`}
                >
                  {tier.cta}
                </Link>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* FAQ */}
      <section className="py-24 md:py-32 border-t border-border/50">
        <div className="mx-auto max-w-2xl px-6">
          <h2 className="text-2xl md:text-3xl font-bold tracking-tight text-center mb-12">
            Frequently asked questions
          </h2>
          <div className="border border-border rounded-xl px-6">
            {faqs.map((faq) => (
              <FAQ key={faq.q} q={faq.q} a={faq.a} />
            ))}
          </div>
        </div>
      </section>

      {/* Bottom CTA */}
      <section className="py-24 border-t border-border/50">
        <div className="mx-auto max-w-4xl px-6 text-center">
          <h2 className="text-2xl md:text-3xl font-bold tracking-tight">
            Start tunneling in seconds
          </h2>
          <p className="mt-4 text-muted">
            No credit card required. Free forever for personal use.
          </p>
          <Link
            href="/docs"
            className="mt-8 inline-flex h-11 items-center gap-2 rounded-lg bg-accent px-6 text-sm font-medium text-black hover:bg-accent/90 transition-colors"
          >
            Get Started Free
            <ArrowRight className="w-4 h-4" />
          </Link>
        </div>
      </section>
    </div>
  );
}
