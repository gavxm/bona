export function IconClose({ size = 14 }: { size?: number }) {
  return (
    <svg width={size} height={size} viewBox="0 0 14 14" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round">
      <path d="M3 3l8 8M11 3l-8 8" />
    </svg>
  );
}

export function IconCheck({ size = 10, className }: { size?: number; className?: string }) {
  return (
    <svg width={size} height={size} viewBox="0 0 10 10" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" className={className}>
      <path d="M2.5 5.5l2 2L7.5 3" />
    </svg>
  );
}

export function IconChevronLeft({ size = 10 }: { size?: number }) {
  return (
    <svg width={size} height={size} viewBox="0 0 10 10" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round">
      <path d="M6.5 1.5L3 5l3.5 3.5" />
    </svg>
  );
}

export function IconChevronRight({ size = 10 }: { size?: number }) {
  return (
    <svg width={size} height={size} viewBox="0 0 10 10" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round">
      <path d="M3.5 1.5L7 5l-3.5 3.5" />
    </svg>
  );
}

export function IconLock({ size = 10, className }: { size?: number; className?: string }) {
  return (
    <svg width={size} height={size} viewBox="0 0 10 10" fill="none" stroke="currentColor" strokeWidth="1.2" strokeLinecap="round" strokeLinejoin="round" className={className}>
      <rect x="2" y="5" width="6" height="4" rx="0.5" />
      <path d="M3.5 5V3.5a1.5 1.5 0 0 1 3 0V5" />
    </svg>
  );
}

export function IconDownload({ size = 14, className }: { size?: number; className?: string }) {
  return (
    <svg width={size} height={size} viewBox="0 0 14 14" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" className={className}>
      <path d="M7 2v7.5M3.5 7L7 10.5 10.5 7M3 12.5h8" />
    </svg>
  );
}

export function IconChevronDown({ size = 10, className }: { size?: number; className?: string }) {
  return (
    <svg width={size} height={size} viewBox="0 0 10 10" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" className={className}>
      <path d="M2 3.5L5 7l3-3.5" />
    </svg>
  );
}

export function IconAlert({ size = 14, className }: { size?: number; className?: string }) {
  return (
    <svg width={size} height={size} viewBox="0 0 14 14" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" className={className}>
      <path d="M7 5v2.5M7 9.5v0" />
      <path d="M6.13 2.26L1.52 10.5a1 1 0 0 0 .87 1.5h9.22a1 1 0 0 0 .87-1.5L7.87 2.26a1 1 0 0 0-1.74 0z" />
    </svg>
  );
}

export function IconArrowRight({ size = 11, className }: { size?: number; className?: string }) {
  return (
    <svg width={size} height={size} viewBox="0 0 11 11" fill="none" stroke="currentColor" strokeWidth="1.3" strokeLinecap="round" strokeLinejoin="round" className={className}>
      <path d="M2 5.5h7M6.5 3L9 5.5 6.5 8" />
    </svg>
  );
}

export function IconVerifiedSeal({ size = 14 }: { size?: number }) {
  return (
    <svg width={size} height={size} viewBox="0 0 14 14" fill="none" className="shrink-0">
      <circle cx="7" cy="7" r="6" stroke="currentColor" strokeWidth="1" className="text-status-ok" />
      <circle cx="7" cy="7" r="4" stroke="currentColor" strokeWidth="0.5" className="text-status-ok opacity-50" />
      <path d="M5 7l1.5 1.5L9.5 5" stroke="currentColor" strokeWidth="1" strokeLinecap="round" strokeLinejoin="round" className="text-status-ok" />
    </svg>
  );
}
