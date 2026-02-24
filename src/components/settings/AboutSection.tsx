import { useTranslation } from "react-i18next";

export default function AboutSection() {
  const { t } = useTranslation("settings");

  return (
    <div className="flex flex-col gap-3">
      <h3 className="text-[15px] font-semibold mb-1" style={{ color: "var(--app-text-primary)" }}>
        {t("aboutTitle")}
      </h3>

      <div className="flex flex-col gap-2">
        {([
          [t("appName"), "CC-Panes"],
          [t("version"), "0.1.0"],
          [t("description"), t("appDescription")],
          [t("techStack"), "Tauri 2 + React 19 + TypeScript"],
        ] as const).map(([label, value]) => (
          <div
            key={label}
            className="flex justify-between items-center py-1.5"
            style={{ borderBottom: "1px solid var(--app-border)" }}
          >
            <span className="text-[13px]" style={{ color: "var(--app-text-secondary)" }}>{label}</span>
            <span className="text-[13px] font-medium" style={{ color: "var(--app-text-primary)" }}>{value}</span>
          </div>
        ))}
      </div>
    </div>
  );
}
