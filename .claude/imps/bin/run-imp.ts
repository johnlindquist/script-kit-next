#!/usr/bin/env bun
import { runImp } from "../lib/isolated.ts";
import { makeProjectImpConfig } from "../lib/project-config.ts";

export const config = makeProjectImpConfig();

if (import.meta.main) await runImp(config);
