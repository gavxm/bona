type IdleStyle = "traverse" | "pulse" | "orbit" | "scan";

export function LogoMark({ animate = false, idle = "scan" }: { animate?: boolean; idle?: IdleStyle }) {
  const outlines = (
    <>
      <circle cx="22" cy="32" r="20" stroke="#FFFFFF" strokeWidth="3.6" />
      <circle cx="44" cy="32" r="20" stroke="#FFFFFF" strokeWidth="3.6" />
      <circle cx="66" cy="32" r="20" stroke="#FFFFFF" strokeWidth="3.6" />
      <circle cx="88" cy="32" r="20" stroke="#FFFFFF" strokeWidth="3.6" />
    </>
  );

  // Loading: fast traversal
  if (animate) {
    return (
      <svg width="112" height="64" viewBox="0 0 112 64" fill="none" xmlns="http://www.w3.org/2000/svg" className="mx-auto">
        {outlines}
        <circle cx="22" cy="32" r="20" fill="#FFFFFF">
          <animate attributeName="cx" values="22;44;66;88;88;66;44;22" dur="2s" repeatCount="indefinite"
            keyTimes="0;0.15;0.3;0.45;0.55;0.7;0.85;1" calcMode="spline"
            keySplines="0.4 0 0.2 1;0.4 0 0.2 1;0.4 0 0.2 1;0.4 0 0.2 1;0.4 0 0.2 1;0.4 0 0.2 1;0.4 0 0.2 1" />
        </circle>
      </svg>
    );
  }

  return (
    <svg width="112" height="64" viewBox="0 0 112 64" fill="none" xmlns="http://www.w3.org/2000/svg" className="mx-auto">
      {outlines}

      {idle === "traverse" && (
        <circle cx="88" cy="32" r="20" fill="#FFFFFF">
          <animate attributeName="cx" values="22;44;66;88;88;66;44;22" dur="4s" repeatCount="indefinite"
            keyTimes="0;0.15;0.3;0.45;0.55;0.7;0.85;1" calcMode="spline"
            keySplines="0.4 0 0.2 1;0.4 0 0.2 1;0.4 0 0.2 1;0.4 0 0.2 1;0.4 0 0.2 1;0.4 0 0.2 1;0.4 0 0.2 1" />
        </circle>
      )}

      {idle === "pulse" && (
        <circle cx="88" cy="32" r="20" fill="#FFFFFF">
          <animate attributeName="r" values="20;17;20" dur="3s" repeatCount="indefinite" calcMode="spline"
            keySplines="0.4 0 0.6 1;0.4 0 0.6 1" />
          <animate attributeName="opacity" values="1;0.6;1" dur="3s" repeatCount="indefinite" calcMode="spline"
            keySplines="0.4 0 0.6 1;0.4 0 0.6 1" />
        </circle>
      )}

      {idle === "orbit" && (
        <circle cx="55" cy="32" r="20" fill="#FFFFFF">
          <animate attributeName="cx" values="88;66;22;44;88" dur="6s" repeatCount="indefinite" calcMode="spline"
            keySplines="0.5 0 0.5 1;0.5 0 0.5 1;0.5 0 0.5 1;0.5 0 0.5 1" />
          <animate attributeName="cy" values="32;22;32;42;32" dur="6s" repeatCount="indefinite" calcMode="spline"
            keySplines="0.5 0 0.5 1;0.5 0 0.5 1;0.5 0 0.5 1;0.5 0 0.5 1" />
        </circle>
      )}

      {idle === "scan" && [22, 44, 66, 88].map((cx, i) => (
        <circle key={cx} cx={cx} cy={32} r="20" fill="#FFFFFF">
          <animate attributeName="opacity" values="0;0;1;1;0;0" dur="4s" repeatCount="indefinite"
            begin={`${i * 0.5}s`} keyTimes="0;0.05;0.15;0.5;0.6;1" calcMode="spline"
            keySplines="0.4 0 0.6 1;0.4 0 0.6 1;0.4 0 0.6 1;0.4 0 0.6 1;0.4 0 0.6 1" />
        </circle>
      ))}
    </svg>
  );
}
