import { describe, expect, test } from "bun:test";
import { compareDesignFidelity } from "./design-fidelity";

const manifest = (extra={}) => ({ screen:{id:"launcher",target:{automationId:"main"},viewport:{width:100,height:80}}, elements:[{fidelityId:"title",gpuiId:"title",exactText:true}], ...extra });
const dom = (rect={x:12,y:23,width:30,height:10}, elements:any[]|null=null) => ({ screenId:"launcher",viewport:{width:100,height:80},windowRect:{x:10,y:20,width:100,height:80},devicePixelRatio:2,elements:elements ?? [{fidelityId:"title",rect,visibleRect:rect,clipRect:{x:10,y:20,width:100,height:80},text:" Hello   world ",style:{},overflow:{x:false,y:false,truncated:false}}] });
const gpui = (rect={x:102,y:203,width:30,height:10}, elements:any[]|null=null) => ({ resolvedTarget:{automationId:"main"},windowRect:{x:100,y:200,width:100,height:80},elements:elements ?? [{semanticId:"title",bounds:rect,visibleBounds:rect,clipBounds:{x:100,y:200,width:100,height:80},text:"Hello world",measurementProvenance:"paint-time",measurementFrameGeneration:7}] });

describe("design fidelity comparator", () => {
  test("exact green", () => { const r=compareDesignFidelity(manifest(),gpui(),dom()); expect(r.classification).toBe("ok"); expect(r.elements[0].iou).toBe(1); });
  test("within-tolerance green", () => { const r=compareDesignFidelity(manifest(),gpui({x:102.5,y:203,width:30,height:10}),dom()); expect(r.classification).toBe("ok"); expect(r.elements[0].deltas.x).toBe(.5); });
  test(">0.5 red", () => { const r=compareDesignFidelity(manifest(),gpui({x:102.51,y:203,width:30,height:10}),dom()); expect(r.classification).toBe("reproduced"); expect(r.elements[0].withinTolerance).toBe(false); });
  test("missing DOM red", () => { expect(compareDesignFidelity(manifest(),gpui(),dom(undefined,[])).errors).toContain("DOM mapping title resolved 0 times"); });
  test("missing GPUI red", () => { expect(compareDesignFidelity(manifest(),gpui(undefined,[]),dom()).errors).toContain("GPUI mapping title resolved 0 times"); });
  test("duplicate mapping red", () => { const m=manifest({elements:[{fidelityId:"title",gpuiId:"title"},{fidelityId:"title",gpuiId:"other"}]}); expect(compareDesignFidelity(m,gpui(),dom()).errors).toContain("Manifest mappings must be one-to-one"); });
  test("formula-only provenance red", () => { const g=gpui(undefined,[{semanticId:"title",bounds:{x:102,y:203,width:30,height:10},text:"Hello world",measurementProvenance:"formula-derived"}]); expect(compareDesignFidelity(manifest({requirePaintMeasurement:true}),g,dom()).errors).toContain("Paint-time GPUI measurement required for title"); });
  test("viewport mismatch red", () => { const d={...dom(),viewport:{width:99,height:80}}; expect(compareDesignFidelity(manifest(),gpui(),d).errors).toContain("Viewport mismatch: manifestVsDom"); });
  test("optional image-diff is supplemental evidence", () => { const r=compareDesignFidelity(manifest(),gpui(),dom(),{tool:"script-kit-devtools.image-diff",classification:"ok",changedPixelCount:42}); expect(r.classification).toBe("ok"); expect(r.imageDiffEvidence?.changedPixelCount).toBe(42); expect(r.warnings[0]).toContain("supplemental"); });
  test("required image diff is fail-closed", () => { const r=compareDesignFidelity(manifest({imageDiff:{required:true,maxChangedPixelRatio:0}}),gpui(),dom()); expect(r.errors).toContain("A valid image-diff receipt is required"); });
  test("required image diff enforces changed-pixel threshold", () => { const r=compareDesignFidelity(manifest({imageDiff:{required:true,maxChangedPixelRatio:0,requireSameSize:true}}),gpui(),dom(),{tool:"script-kit-devtools.image-diff",classification:"ok",changedPixelRatio:.001,dimensions:{sameSize:true}}); expect(r.errors).toContain("Changed-pixel ratio exceeds the manifest limit"); });
  test("required image diff rejects stale capture hashes", () => { const r=compareDesignFidelity(manifest({imageDiff:{required:true,maxChangedPixelRatio:0,requireInputHashes:true}}),gpui(),dom(),{tool:"script-kit-devtools.image-diff",classification:"ok",changedPixelRatio:0,inputHashes:{red:{matchesReceipt:true},green:{matchesReceipt:false}}}); expect(r.errors).toContain("Both image inputs must match their capture receipt hashes"); });
  test("required image diff rejects blocked receipts", () => { const r=compareDesignFidelity(manifest({imageDiff:{required:true,maxChangedPixelRatio:0}}),gpui(),dom(),{tool:"script-kit-devtools.image-diff",classification:"blocked",changedPixelRatio:0}); expect(r.errors).toContain("A valid image-diff receipt is required"); });
  test("required GPUI raster rejects non-OS image evidence", () => { const r=compareDesignFidelity(manifest({imageDiff:{required:true,maxChangedPixelRatio:0,requireRedOsEvidence:true}}),gpui(),dom(),{tool:"script-kit-devtools.image-diff",classification:"ok",changedPixelRatio:0,inputEvidence:{red:{source:"app-render",classification:"captured",countsAsOsScreenshotEvidence:false}}}); expect(r.errors).toContain("The GPUI image must be a nonblank OS compositor capture"); });
  test("unmapped marked DOM node is red", () => { const d=dom(undefined,[...dom().elements,{fidelityId:"extra",rect:{x:0,y:0,width:1,height:1},text:"",style:{},overflow:{}}]); expect(compareDesignFidelity(manifest(),gpui(),d).errors).toContain("Every data-fidelity-id must be mapped"); });
  test("accepts real layout.ts window-relative node receipts", () => {
    const receipt={tool:"script-kit-devtools.layout",target:{automationId:"main"},windowRect:{x:100,y:200,width:100,height:80},nodes:[{name:"title",bounds:{x:2,y:3,width:30,height:10},visibleBounds:{x:2,y:3,width:30,height:10},text:"Hello world",measurementProvenance:"paint-time",coordinateSpace:"window",measurementFrameGeneration:7}]};
    const r=compareDesignFidelity(manifest({requirePaintMeasurement:true,requireVisibleMeasurement:true}),receipt,dom());
    expect(r.classification).toBe("ok");
    expect(r.elements[0].gpuiRect).toEqual({x:2,y:3,right:32,bottom:13,width:30,height:10});
  });
  test("rejects clipped visible geometry drift even when full bounds match", () => {
    const g=gpui(undefined,[{semanticId:"title",bounds:{x:102,y:203,width:30,height:10},visibleBounds:{x:102,y:205,width:30,height:8},text:"Hello world",measurementProvenance:"paint-time",measurementFrameGeneration:7}]);
    const r=compareDesignFidelity(manifest({requireVisibleMeasurement:true}),g,dom());
    expect(r.classification).toBe("reproduced");
    expect(r.errors).toContain("Visible geometry exceeds tolerance for title");
  });
  test("rejects paint measurements mixed across rendered frames", () => {
    const twoElementManifest=manifest({requirePaintMeasurement:true,elements:[{fidelityId:"title",gpuiId:"title"},{fidelityId:"subtitle",gpuiId:"subtitle"}]});
    const twoElementDom=dom(undefined,[
      {fidelityId:"title",rect:{x:12,y:23,width:30,height:10},text:"",style:{},overflow:{}},
      {fidelityId:"subtitle",rect:{x:12,y:34,width:30,height:10},text:"",style:{},overflow:{}},
    ]);
    const receipt={tool:"script-kit-devtools.layout",target:{automationId:"main"},windowRect:{x:100,y:200,width:100,height:80},nodes:[
      {name:"title",bounds:{x:2,y:3,width:30,height:10},measurementProvenance:"paint-time",coordinateSpace:"window",measurementFrameGeneration:7},
      {name:"subtitle",bounds:{x:2,y:14,width:30,height:10},measurementProvenance:"paint-time",coordinateSpace:"window",measurementFrameGeneration:8},
    ]};
    const r=compareDesignFidelity(twoElementManifest,receipt,twoElementDom);
    expect(r.classification).toBe("reproduced");
    expect(r.errors).toContain("All mapped GPUI elements must come from one completed paint frame");
  });
});
