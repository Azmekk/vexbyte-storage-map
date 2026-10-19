import type { OperatingSystem } from "@/lib/backend/platform";

/**
 * Whether the top bar offers the restart as administrator. Only Windows can restart itself
 * elevated, and only an instance known to run without admin rights needs it.
 */
export function shouldOfferAdminRestart(
  operatingSystem: OperatingSystem,
  isElevated: boolean | null,
): boolean {
  return operatingSystem === "windows" && isElevated === false;
}
