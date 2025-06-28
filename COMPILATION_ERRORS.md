# StepheyBot Music Compilation Errors

This document organizes all compilation errors that need to be fixed systematically.

## Error Categories

### 1. E0716: Temporary Value Dropped While Borrowed

#### musicbrainz.rs
- **Line 345:34** - `&inc.join("+")` - temporary value freed while still in use
- **Line 381:34** - `&inc.join("+")` - temporary value freed while still in use  
- **Line 417:34** - `&inc.join("+")` - temporary value freed while still in use
- **Line 449:36** - `&limit.to_string()` - temporary value freed while still in use
- **Line 480:36** - `&limit.to_string()` - temporary value freed while still in use
- **Line 515:36** - `&limit.to_string()` - temporary value freed while still in use

#### navidrome.rs
- **Line 645:35** - `&t.to_string()` - temporary value freed while still in use

**Fix Strategy**: Create intermediate variables to store the values before borrowing them.

### 2. E0382: Borrow of Moved Value

#### musicbrainz.rs
- **Line 592:12** - `images` moved in for loop, then borrowed - use `&images` in for loop

#### qbittorrent.rs
- **Line 227:17** - `response` moved and then borrowed - need to handle response properly

**Fix Strategy**: Use references in loops or restructure to avoid moving values before borrowing.

### 3. E0597: Borrowed Value Does Not Live Long Enough

#### navidrome.rs
- **Line 413:34** - `s_str` doesn't live long enough
- **Line 417:36** - `o_str` doesn't live long enough
- **Line 481:41** - `ac_str` doesn't live long enough
- **Line 485:40** - `alc_str` doesn't live long enough
- **Line 489:39** - `sc_str` doesn't live long enough

**Fix Strategy**: Restructure to create longer-lived bindings or use different approach for parameter building.

### 4. E0283: Type Annotations Needed

#### navidrome.rs
- **Line 336:13** - `SubsonicResponse<_>` needs type annotation
- **Line 797:12** - Generic type parameter needs specification

**Fix Strategy**: Add explicit type annotations or restructure method calls.

### 5. E0308: Mismatched Types

#### navidrome.rs
- **Line 449:35** - Expected `&String`, found `&str`
- **Line 461:49** - Expected `&[(&str, &str)]`, found `&Vec<(&str, &String)>`

**Fix Strategy**: Fix type mismatches by converting between String/str types.

### 6. E0599: No Method Found

#### services/mod.rs
- **Line 114:53** - `Arc<RecommendationService>` missing `health_check` method
- **Line 133:56** - `Arc<RecommendationService>` missing `get_stats` method  
- **Line 160:45** - `Arc<RecommendationService>` missing `shutdown` method

**Fix Strategy**: Implement the `Service` trait for `RecommendationService` or provide stub implementations.

### 7. E0733: Recursion in Async Function

#### services/library.rs
- **Line 123:5** - Recursive async function requires boxing

**Fix Strategy**: Use `Box::pin()` for recursive calls or restructure to avoid recursion.

## Priority Fix Order

1. **High Priority - Core Download Functionality**
   - Fix qbittorrent.rs errors (affects download service)
   - Fix services/mod.rs trait implementations
   
2. **Medium Priority - API Functionality**  
   - Fix navidrome.rs type and lifetime issues
   - Fix musicbrainz.rs temporary value issues
   
3. **Low Priority - Supporting Features**
   - Fix library.rs recursive function
   - Clean up unused imports (warnings)

## Files Requiring Fixes

1. `src/clients/qbittorrent.rs` - 1 error
2. `src/clients/navidrome.rs` - 8 errors  
3. `src/clients/musicbrainz.rs` - 7 errors
4. `src/services/mod.rs` - 3 errors
5. `src/services/library.rs` - 1 error

Total: **20 compilation errors** to fix

## Next Steps

Work through errors in priority order:
1. Fix qbittorrent.rs response handling
2. Implement missing Service trait methods  
3. Fix navidrome.rs type annotations and lifetimes
4. Fix musicbrainz.rs temporary value issues
5. Fix library.rs recursive async function