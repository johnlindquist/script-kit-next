// Compiled into the private scratch directory only AFTER hosted-run attestation.
// No capture, UI scripting or Apple Events. stdout is a small non-sensitive tuple.
#import <AppKit/AppKit.h>
#import <CoreGraphics/CoreGraphics.h>
#import <Security/Security.h>
#import <libproc.h>
#import <pwd.h>
#import <sys/stat.h>
#import <unistd.h>

static BOOL fdMatches(pid_t pid, int fd, const char *path) {
    struct vnode_fdinfowithpath v = {0};
    struct stat s;
    return stat(path, &s) == 0 &&
           proc_pidfdinfo(pid, fd, PROC_PIDFDVNODEPATHINFO, &v, sizeof v) == sizeof v &&
           v.pvip.vip_vi.vi_stat.vst_dev == s.st_dev && v.pvip.vip_vi.vi_stat.vst_ino == s.st_ino;
}
static BOOL aqua(BOOL report) {
    NSDictionary *session = CFBridgingRelease(CGSessionCopyCurrentDictionary());
    struct stat s;
    BOOL onConsole = [session[(__bridge NSString *)kCGSessionOnConsoleKey] boolValue];
    BOOL loginDone = [session[(__bridge NSString *)kCGSessionLoginDoneKey] boolValue];
    BOOL locked = [session[@"CGSSessionScreenIsLocked"] boolValue];
    BOOL runner = [session[(__bridge NSString *)kCGSessionUserNameKey] isEqual:@"runner"];
    BOOL consoleOwner = stat("/dev/console", &s) == 0 && s.st_uid == getuid();
    BOOL display = CGDisplayIsActive(CGMainDisplayID());
    BOOL ready = session && onConsole && loginDone && !locked && runner && consoleOwner && display;
    if (report) {
        NSDictionary *facts = @{@"scope":@"host_preflight_only", @"session":@(session != nil), @"onConsole":@(onConsole), @"loginDone":@(loginDone), @"locked":@(locked), @"runnerSession":@(runner), @"consoleOwner":@(consoleOwner), @"activeDisplay":@(display), @"aqua":@(ready)};
        NSData *encoded = [NSJSONSerialization dataWithJSONObject:facts options:0 error:nil];
        if (encoded) puts([[[NSString alloc] initWithData:encoded encoding:NSUTF8StringEncoding] UTF8String]);
    }
    return ready;
}
static NSArray *apps(void) {
    return [NSRunningApplication runningApplicationsWithBundleIdentifier:@"com.scriptkit.app"];
}
int main(int argc, char **argv) {
    @autoreleasepool {
        struct passwd *pw = getpwuid(geteuid());
        if (!pw || strcmp(pw->pw_name, "runner") || strcmp(pw->pw_dir, "/Users/runner") ||
            !getenv("SMOKE_CAP") || strcmp(getenv("SMOKE_CAP"), SMOKE_CAPABILITY) || argc < 2)
            return 1;
        if (!strcmp(argv[1], "aqua") || !strcmp(argv[1], "aqua-info"))
            return aqua(!strcmp(argv[1], "aqua-info")) ? 0 : 2;
        if (!strcmp(argv[1], "departed")) {
            if (argc != 4)
                return 12;
            pid_t pid = (pid_t)strtol(argv[2], NULL, 10);
            if (pid < 2)
                return 12;
            struct proc_bsdinfo b = {0};
            if (proc_pidinfo(pid, PROC_PIDTBSDINFO, 0, &b, sizeof b) != sizeof b) {
                if (kill(pid, 0) == -1 && errno == ESRCH) {
                    puts("true");
                    return 0;
                }
                return 12;
            }
            NSString *generation =
                [NSString stringWithFormat:@"%llu.%06llu", b.pbi_start_tvsec, b.pbi_start_tvusec];
            puts([generation isEqual:@(argv[3])] ? "false" : "true");
            return 0;
        }
        NSArray *found = apps();
        if (!strcmp(argv[1], "absent"))
            return found.count == 0 ? 0 : 3;
        if (argc != 5 && argc != 7)
            return 4;
        if (!found.count) {
            puts("null");
            return 0;
        }
        if (found.count != 1)
            return 5;
        NSRunningApplication *app = found.firstObject;
        pid_t pid = app.processIdentifier;
        const char *path = "/Applications/Script Kit.app/Contents/MacOS/script-kit-gpui";
        char actual[PROC_PIDPATHINFO_MAXSIZE] = {0};
        struct proc_bsdinfo b = {0};
        if (pid < 2 || proc_pidpath(pid, actual, sizeof actual) <= 0 || strcmp(path, actual) ||
            ![app.bundleURL.path isEqual:@"/Applications/Script Kit.app"] ||
            ![app.executableURL.path isEqual:@(path)] ||
            proc_pidinfo(pid, PROC_PIDTBSDINFO, 0, &b, sizeof b) != sizeof b || b.pbi_uid != getuid() ||
            !b.pbi_start_tvsec || !fdMatches(pid, 0, argv[2]) || !fdMatches(pid, 1, argv[3]))
            return 6;
        // Dynamic code object, not just the file now occupying its pathname.
        SecCodeRef guest = NULL;
        SecRequirementRef req = NULL;
        CFDictionaryRef info = NULL;
        CFStringRef requirement = CFSTR(
            "anchor apple generic and identifier \"com.scriptkit.app\" and certificate leaf[subject.OU] = "
            "\"9822B7V7MD\" and certificate leaf[field.1.2.840.113635.100.6.1.13] exists");
        if (SecCodeCopyGuestWithAttributes(
                NULL,
                (__bridge CFDictionaryRef)
                    @{(__bridge NSString *)kSecGuestAttributePid : @(pid)},
                0, &guest) ||
            SecRequirementCreateWithString(requirement, 0, &req) || SecCodeCheckValidity(guest, 0, req) ||
            SecCodeCopySigningInformation(guest, kSecCSSigningInformation, &info))
            return 7;
        NSData *digest = ((__bridge NSDictionary *)info)[(__bridge NSString *)kSecCodeInfoUnique];
        NSMutableString *cdhash = [NSMutableString string];
        const unsigned char *bytes = digest.bytes;
        for (NSUInteger i = 0; i < digest.length; ++i)
            [cdhash appendFormat:@"%02x", bytes[i]];
        if (digest.length == 0 || strcmp(cdhash.UTF8String, argv[4]))
            return 8;
        NSString *generation =
            [NSString stringWithFormat:@"%llu.%06llu", b.pbi_start_tvsec, b.pbi_start_tvusec];
        if (!strcmp(argv[1], "terminate")) {
            if (argc != 7 || ![generation isEqual:@(argv[6])] || ![[@(pid) stringValue] isEqual:@(argv[5])])
                return 9;
            // NSRunningApplication retains instance identity across PID recycling.
            return app.terminate ? 0 : 10;
        }
        if (strcmp(argv[1], "inspect"))
            return 11;
        BOOL onscreen = NO;
        for (NSDictionary *w in CFBridgingRelease(
                 CGWindowListCopyWindowInfo(kCGWindowListOptionOnScreenOnly, kCGNullWindowID))) {
            CGRect r;
            if ([w[(__bridge NSString *)kCGWindowOwnerPID] intValue] == pid &&
                [w[(__bridge NSString *)kCGWindowLayer] intValue] >= 0 &&
                CGRectMakeWithDictionaryRepresentation(
                    (__bridge CFDictionaryRef)w[(__bridge NSString *)kCGWindowBounds], &r) &&
                r.size.width > 100 && r.size.height > 100)
                onscreen = YES;
        }
        printf("{\"pid\":%d,\"generation\":\"%s\",\"cdhash\":\"%s\",\"onscreen\":%s}\n", pid,
               generation.UTF8String, cdhash.UTF8String, onscreen ? "true" : "false");
        return 0;
    }
}
