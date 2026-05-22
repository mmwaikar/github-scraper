# C# Interop Guide

This crate exports a C ABI from `cdylib` and can be consumed with P/Invoke.

## Build The Shared Library

```bash
cargo build --release
```

On Linux the library file will be:

- `target/release/libgithub_scraper.so`

## Exported FFI API

- `get_github_repo_stats(const char* url) -> GitHubRepoStatsFfi`
- `free_github_repo_stats(GitHubRepoStatsFfi stats)`

`GitHubRepoStatsFfi` fields:

- `stars`, `watching`, `forks`, `releases`, `used`, `contributors`: UTF-8 C strings (`char*`)
- `error`: UTF-8 C string (`char*`), set when `success == 0`
- `success`: `1` on success, `0` on failure

All returned pointers are owned by Rust and must be released by calling `free_github_repo_stats`.

## C# Example (.NET 8+)

```csharp
using System;
using System.Runtime.InteropServices;

internal static class Native
{
    private const string LibName = "github_scraper";

    [StructLayout(LayoutKind.Sequential)]
    internal struct GitHubRepoStatsFfi
    {
        public IntPtr stars;
        public IntPtr watching;
        public IntPtr forks;
        public IntPtr releases;
        public IntPtr used;
        public IntPtr contributors;
        public IntPtr error;
        public int success;
    }

    [LibraryImport(LibName, EntryPoint = "get_github_repo_stats", StringMarshalling = StringMarshalling.Utf8)]
    internal static partial GitHubRepoStatsFfi GetGitHubRepoStats(string url);

    [LibraryImport(LibName, EntryPoint = "free_github_repo_stats")]
    internal static partial void FreeGitHubRepoStats(GitHubRepoStatsFfi stats);

    internal static string PtrToStringUtf8OrEmpty(IntPtr ptr)
        => ptr == IntPtr.Zero ? string.Empty : Marshal.PtrToStringUTF8(ptr) ?? string.Empty;
}

public class Program
{
    public static void Main()
    {
        var stats = Native.GetGitHubRepoStats("https://github.com/rust-lang/rust");

        try
        {
            if (stats.success == 0)
            {
                Console.WriteLine($"Error: {Native.PtrToStringUtf8OrEmpty(stats.error)}");
                return;
            }

            Console.WriteLine($"Stars: {Native.PtrToStringUtf8OrEmpty(stats.stars)}");
            Console.WriteLine($"Forks: {Native.PtrToStringUtf8OrEmpty(stats.forks)}");
            Console.WriteLine($"Watching: {Native.PtrToStringUtf8OrEmpty(stats.watching)}");
            Console.WriteLine($"Releases: {Native.PtrToStringUtf8OrEmpty(stats.releases)}");
            Console.WriteLine($"Used by: {Native.PtrToStringUtf8OrEmpty(stats.used)}");
            Console.WriteLine($"Contributors: {Native.PtrToStringUtf8OrEmpty(stats.contributors)}");
        }
        finally
        {
            Native.FreeGitHubRepoStats(stats);
        }
    }
}
```

## Runtime Loading Notes

- Linux: ensure `libgithub_scraper.so` is in `LD_LIBRARY_PATH` or next to the app.
- Windows: the file name is `github_scraper.dll`.
- macOS: the file name is `libgithub_scraper.dylib`.
