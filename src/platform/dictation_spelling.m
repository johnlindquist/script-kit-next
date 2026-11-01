// Read-only macOS providers for one capture. No activation, disk, clipboard,
// permission requests or logging. All returned strings are bounded UTF-8.
#import <AppKit/AppKit.h>
#import <ApplicationServices/ApplicationServices.h>
#import <Carbon/Carbon.h>
#import <NaturalLanguage/NaturalLanguage.h>

extern AXError _AXUIElementGetWindow(AXUIElementRef, CGWindowID *) __attribute__((weak_import));

static id attr(AXUIElementRef el, CFStringRef key) {
    CFTypeRef value = NULL;
    AXUIElementCopyAttributeValue(el, key, &value);
    return CFBridgingRelease(value);
}
static bool safeField(AXUIElementRef el) {
    // No subrole is normal in editors. An AX messaging failure is not.
    for (NSString *key in @[@"AXSubrole", @"AXProtectedContent"]) {
        CFTypeRef v = NULL;
        AXError err = AXUIElementCopyAttributeValue(el, (__bridge CFStringRef)key, &v);
        id value = CFBridgingRelease(v);
        if (err == kAXErrorNoValue || err == kAXErrorAttributeUnsupported) continue;
        if (err != kAXErrorSuccess || !value) return false;
        if ([key isEqual:@"AXSubrole"]) {
            if (![value isKindOfClass:NSString.class] || [value isEqual:@"AXSecureTextField"]) return false;
        } else if (![value isKindOfClass:NSNumber.class] || [value boolValue]) return false;
    }
    return true;
}
static id originalWindow(int pid, uint32_t wid, AXUIElementRef original) {
    if (!original || !_AXUIElementGetWindow || !AXIsProcessTrusted() || IsSecureEventInputEnabled() ||
        NSWorkspace.sharedWorkspace.frontmostApplication.processIdentifier != pid) return nil;
    id app = CFBridgingRelease(AXUIElementCreateApplication(pid));
    if (!app) return nil;
    AXUIElementSetMessagingTimeout((__bridge AXUIElementRef)app, 0.08f);
    id current = attr((__bridge AXUIElementRef)app, kAXFocusedUIElementAttribute);
    if (!current || !CFEqual((__bridge CFTypeRef)current, original) || !safeField(original)) return nil;
    id window = attr(original,kAXWindowAttribute);
    CGWindowID actual = 0;
    if (!window || _AXUIElementGetWindow((__bridge AXUIElementRef)window,&actual) != kAXErrorSuccess || actual != wid) return nil;
    return window;
}
bool sk_spelling_original(int pid, uint32_t wid, AXUIElementRef field) {
    @autoreleasepool { return originalWindow(pid,wid,field) != nil; }
}
static void append(NSMutableString *out, id value) {
    if (![value isKindOfClass:NSString.class] || [value length] > 2048 || ![value length] || out.length + [value length] + 1 > 4096) return;
    [out appendFormat:@"%@\n",value];
}
static bool rangeAttr(AXUIElementRef field, CFStringRef key, CFRange *r) {
    id v = attr(field,key);
    return v && CFGetTypeID((__bridge CFTypeRef)v) == AXValueGetTypeID() && AXValueGetValue((__bridge AXValueRef)v,kAXValueCFRangeType,r) && r->location >= 0 && r->length >= 0;
}
static void nearCaret(AXUIElementRef field, NSMutableString *out) {
    CFRange view, selection;
    if (!rangeAttr(field,kAXVisibleCharacterRangeAttribute,&view) || !rangeAttr(field,kAXSelectedTextRangeAttribute,&selection) || view.length <= 0 || view.location > LONG_MAX-view.length) return;
    CFIndex end = view.location + view.length;
    CFIndex start = MAX(view.location, MIN(end-1, selection.location-768));
    CFRange bounded = CFRangeMake(start, MIN(1536,end-start));
    id range = CFBridgingRelease(AXValueCreate(kAXValueCFRangeType,&bounded));
    CFTypeRef value = NULL;
    if (range) AXUIElementCopyParameterizedAttributeValue(field,kAXStringForRangeParameterizedAttribute,(__bridge CFTypeRef)range,&value);
    append(out,CFBridgingRelease(value));
}
// Geometry is required for labels reached through AXChildren: some editors
// expose the entire document (including offscreen rows) through that attribute.
static CGRect frame(AXUIElementRef el) {
    id pos = attr(el,kAXPositionAttribute), size = attr(el,kAXSizeAttribute);
    CGPoint p; CGSize s;
    if (!pos || !size || CFGetTypeID((__bridge CFTypeRef)pos)!=AXValueGetTypeID() || CFGetTypeID((__bridge CFTypeRef)size)!=AXValueGetTypeID() || !AXValueGetValue((__bridge AXValueRef)pos,kAXValueCGPointType,&p) || !AXValueGetValue((__bridge AXValueRef)size,kAXValueCGSizeType,&s)) return CGRectNull;
    return (CGRect){p,s};
}
static void visibleLabels(id window, NSMutableString *out) {
    CGRect bounds = frame((__bridge AXUIElementRef)window);
    if (CGRectIsEmpty(bounds)) return;
    NSMutableArray *queue = [NSMutableArray arrayWithObject:window];
    NSTimeInterval deadline = NSProcessInfo.processInfo.systemUptime + 0.075;
    for (NSUInteger i=0; i<queue.count && out.length<4096 && NSProcessInfo.processInfo.systemUptime<deadline; ++i) {
        AXUIElementRef el = (__bridge AXUIElementRef)queue[i];
        NSString *role = attr(el,kAXRoleAttribute);
        if (![role isKindOfClass:NSString.class]) continue;
        if ([role isEqual:@"AXStaticText"] || [role isEqual:@"AXHeading"]) {
            if (CGRectIntersectsRect(bounds,frame(el)) && safeField(el)) append(out,attr(el,kAXValueAttribute));
            continue;
        }
        // Never descend into editable/secure controls or read their values.
        if (![@[@"AXWindow",@"AXGroup",@"AXScrollArea",@"AXWebArea",@"AXSplitGroup",@"AXList",@"AXRow",@"AXTable"] containsObject:role]) continue;
        CFIndex room = MIN((NSUInteger)16,64-queue.count);
        if (!room) continue;
        CFArrayRef next = NULL;
        AXError err = AXUIElementCopyAttributeValues(el,kAXVisibleChildrenAttribute,0,room,&next);
        if (!next && (err == kAXErrorAttributeUnsupported || err == kAXErrorNoValue)) AXUIElementCopyAttributeValues(el,kAXChildrenAttribute,0,room,&next);
        for (id child in CFBridgingRelease(next)) if (queue.count<64 && CFGetTypeID((__bridge CFTypeRef)child)==AXUIElementGetTypeID()) [queue addObject:child];
    }
}
// false means unsafe/unavailable, never "try OCR instead".
bool sk_spelling_excerpt(int pid, uint32_t wid, AXUIElementRef field, char *bytes, size_t cap) {
    @autoreleasepool {
        id window = originalWindow(pid,wid,field);
        if (!window) return false;
        NSMutableString *text = [NSMutableString string];
        nearCaret(field,text);
        append(text,attr((__bridge AXUIElementRef)window,kAXTitleAttribute));
        visibleLabels(window,text);
        return originalWindow(pid,wid,field) && [text getCString:bytes maxLength:cap encoding:NSUTF8StringEncoding];
    }
}
void sk_spelling_names(const char *bytes, char *out, size_t cap) {
    @autoreleasepool {
        NSString *s = [NSString stringWithUTF8String:bytes];
        if (!s.length) return;
        NLTagger *tagger = [[NLTagger alloc] initWithTagSchemes:@[NLTagSchemeNameType]];
        tagger.string=s;
        NSMutableString *names=[NSMutableString string];
        [tagger enumerateTagsInRange:NSMakeRange(0,s.length) unit:NLTokenUnitWord scheme:NLTagSchemeNameType options:NLTaggerOmitWhitespace|NLTaggerOmitPunctuation|NLTaggerJoinNames usingBlock:^(NLTag tag, NSRange range, BOOL *stop){
            if (([tag isEqual:NLTagPersonalName] || [tag isEqual:NLTagOrganizationName] || [tag isEqual:NLTagPlaceName]) && range.length <= 60) append(names,[s substringWithRange:range]);
            *stop=names.length>=512;
        }];
        [names getCString:out maxLength:cap encoding:NSUTF8StringEncoding];
    }
}
