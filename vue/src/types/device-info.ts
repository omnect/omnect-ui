import type { SystemInfo } from "./system-info";
import type { Timeouts } from "./timeouts";

export type DeviceInfoData = {
  systemInfo: SystemInfo | undefined,
  online: boolean,
  factoryResetStatus: string,
  timeouts: Timeouts  | undefined
}