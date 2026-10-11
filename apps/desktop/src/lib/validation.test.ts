import { describe, expect, it } from "vitest";
import { normalizeList, validateDnsServers, validatePort, validateRoutes } from "./validation";

describe("validatePort", () => {
  it("accepts unprivileged ports", () => {
    expect(validatePort("31416")).toBeNull();
    expect(validatePort("1024")).toBeNull();
    expect(validatePort("65535")).toBeNull();
  });

  it("rejects everything else", () => {
    for (const value of ["", "80", "0", "65536", "12.5", "-1", "abc", "1e4"]) {
      expect(validatePort(value), value).not.toBeNull();
    }
  });
});

describe("validateDnsServers", () => {
  it("accepts empty and IPv4 lists", () => {
    expect(validateDnsServers("")).toBeNull();
    expect(validateDnsServers("8.8.8.8, 1.1.1.1")).toBeNull();
  });

  it("rejects anything that is not an IPv4 address", () => {
    for (const value of ["8.8.8", "256.1.1.1", "::1", "8.8.8.8;reboot", "8.8.8.8 1.1.1.1", "01.2.3.4"]) {
      expect(validateDnsServers(value), value).not.toBeNull();
    }
  });
});

describe("validateRoutes", () => {
  it("accepts empty and CIDR lists", () => {
    expect(validateRoutes("")).toBeNull();
    expect(validateRoutes("10.0.0.0/8, 192.168.0.0/16,0.0.0.0/0")).toBeNull();
  });

  it("rejects malformed routes", () => {
    for (const value of ["10.0.0.0", "10.0.0.0/33", "10.0.0.0/8/1", "10.0.0/8", "0.0.0.0/0$(id)"]) {
      expect(validateRoutes(value), value).not.toBeNull();
    }
  });
});

describe("normalizeList", () => {
  it("trims and drops empty items", () => {
    expect(normalizeList(" 1.1.1.1 , ,8.8.8.8 ")).toBe("1.1.1.1,8.8.8.8");
    expect(normalizeList("  ")).toBeNull();
  });
});
