//! The statically defined default dictionary.
//!
//! These tokens are **not discovered by indexing**. They are compiled into the
//! addon (and into the viewer) so a project token can never collide with a C#
//! word, a language construct, a system type or a built-in method.
//!
//! Layout (all values are `g` tokens):
//!
//! | range | contents |
//! |---|---|
//! | `1..=99` | C# words (`public`=1 … `return`=40) and native shorthand constructs (`?:`=77 …) |
//! | `100..=899` | **system types, pre-assigned** — all BCL/Unity/VRC types this build knows |
//! | `900..=999` | default methods on those types |
//! | `1000..` | project definitions, allocated by [`crate::dict::Dictionary`] |
//!
//! The lists are ordered: a type's token is `SYSTEM_TYPE_BASE + index` in
//! [`SYSTEM_TYPES`], a method's is `DEFAULT_METHOD_BASE + index`. That makes
//! "add a system type" a single line at the end of the list, and keeps every
//! existing number stable until the list is reordered. Reordering or changing
//! the fixed ranges requires bumping [`TABLE_REV`], which invalidates dumps
//! written against the old numbering.

use std::sync::OnceLock;

/// What a statically defined token denotes.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DefaultKind {
    Keyword,
    Modifier,
    Type,
    Method,
    /// An operator/syntax feature of C# (`?:`, `??`, `=>`, `?.`, …).
    Construct,
}

impl DefaultKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            DefaultKind::Keyword => "keyword",
            DefaultKind::Modifier => "modifier",
            DefaultKind::Type => "type",
            DefaultKind::Method => "method",
            DefaultKind::Construct => "construct",
        }
    }

    pub fn parse(s: &str) -> Option<DefaultKind> {
        match s {
            "keyword" => Some(DefaultKind::Keyword),
            "modifier" => Some(DefaultKind::Modifier),
            "type" => Some(DefaultKind::Type),
            "method" => Some(DefaultKind::Method),
            "construct" => Some(DefaultKind::Construct),
            _ => None,
        }
    }
}

/// One statically defined token.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct DefaultToken {
    pub token: u32,
    pub name: &'static str,
    pub kind: DefaultKind,
}

/// Bump when the table's *numbering* changes. Adding entries at the end of a
/// list does not need a bump (existing numbers do not move); reordering or
/// changing a base does.
pub const TABLE_REV: u32 = 2;

/// First statically defined system type token.
pub const SYSTEM_TYPE_FIRST: u32 = 100;
pub const SYSTEM_TYPE_BASE: u32 = SYSTEM_TYPE_FIRST;
/// Number of slots reserved for system types.
pub const SYSTEM_TYPE_CAPACITY: u32 = 800;
/// Last statically defined system type token.
pub const SYSTEM_TYPE_LAST: u32 = SYSTEM_TYPE_BASE + SYSTEM_TYPE_CAPACITY - 1;

/// First statically defined default-method token.
pub const DEFAULT_METHOD_FIRST: u32 = 900;
pub const DEFAULT_METHOD_BASE: u32 = DEFAULT_METHOD_FIRST;
pub const DEFAULT_METHOD_CAPACITY: u32 = 100;
/// Last token reserved for the default dictionary; project tokens start after.
pub const DEFAULT_MAX: u32 = 999;

// The owner's explicit modifier numbers, usable directly in code.
pub const PUBLIC: u32 = 1;
pub const PRIVATE: u32 = 2;
pub const STATIC: u32 = 3;
pub const CONST: u32 = 4;
pub const VOID: u32 = 5;
pub const DEFAULT: u32 = 33;
pub const RETURN: u32 = 40;
pub const IS: u32 = 62;
pub const VAR: u32 = 64;

struct LangEntry {
    token: u32,
    name: &'static str,
    kind: DefaultKind,
}

const LANGUAGE: &[LangEntry] = &[
    LangEntry {
        token: 1,
        name: "public",
        kind: DefaultKind::Modifier,
    },
    LangEntry {
        token: 2,
        name: "private",
        kind: DefaultKind::Modifier,
    },
    LangEntry {
        token: 3,
        name: "static",
        kind: DefaultKind::Modifier,
    },
    LangEntry {
        token: 4,
        name: "const",
        kind: DefaultKind::Modifier,
    },
    LangEntry {
        token: 5,
        name: "void",
        kind: DefaultKind::Keyword,
    },
    LangEntry {
        token: 6,
        name: "readonly",
        kind: DefaultKind::Modifier,
    },
    LangEntry {
        token: 7,
        name: "sealed",
        kind: DefaultKind::Modifier,
    },
    LangEntry {
        token: 8,
        name: "abstract",
        kind: DefaultKind::Modifier,
    },
    LangEntry {
        token: 9,
        name: "virtual",
        kind: DefaultKind::Modifier,
    },
    LangEntry {
        token: 10,
        name: "override",
        kind: DefaultKind::Modifier,
    },
    LangEntry {
        token: 11,
        name: "partial",
        kind: DefaultKind::Modifier,
    },
    LangEntry {
        token: 12,
        name: "async",
        kind: DefaultKind::Modifier,
    },
    LangEntry {
        token: 13,
        name: "extern",
        kind: DefaultKind::Modifier,
    },
    LangEntry {
        token: 14,
        name: "unsafe",
        kind: DefaultKind::Modifier,
    },
    LangEntry {
        token: 15,
        name: "volatile",
        kind: DefaultKind::Modifier,
    },
    LangEntry {
        token: 16,
        name: "new",
        kind: DefaultKind::Modifier,
    },
    LangEntry {
        token: 17,
        name: "internal",
        kind: DefaultKind::Modifier,
    },
    LangEntry {
        token: 18,
        name: "protected",
        kind: DefaultKind::Modifier,
    },
    LangEntry {
        token: 19,
        name: "event",
        kind: DefaultKind::Keyword,
    },
    LangEntry {
        token: 20,
        name: "delegate",
        kind: DefaultKind::Keyword,
    },
    LangEntry {
        token: 21,
        name: "operator",
        kind: DefaultKind::Keyword,
    },
    LangEntry {
        token: 22,
        name: "implicit",
        kind: DefaultKind::Keyword,
    },
    LangEntry {
        token: 23,
        name: "explicit",
        kind: DefaultKind::Keyword,
    },
    LangEntry {
        token: 24,
        name: "params",
        kind: DefaultKind::Modifier,
    },
    LangEntry {
        token: 25,
        name: "ref",
        kind: DefaultKind::Modifier,
    },
    LangEntry {
        token: 26,
        name: "out",
        kind: DefaultKind::Modifier,
    },
    LangEntry {
        token: 27,
        name: "in",
        kind: DefaultKind::Modifier,
    },
    LangEntry {
        token: 28,
        name: "this",
        kind: DefaultKind::Keyword,
    },
    LangEntry {
        token: 29,
        name: "base",
        kind: DefaultKind::Keyword,
    },
    LangEntry {
        token: 30,
        name: "null",
        kind: DefaultKind::Keyword,
    },
    LangEntry {
        token: 31,
        name: "true",
        kind: DefaultKind::Keyword,
    },
    LangEntry {
        token: 32,
        name: "false",
        kind: DefaultKind::Keyword,
    },
    LangEntry {
        token: 33,
        name: "default",
        kind: DefaultKind::Keyword,
    },
    LangEntry {
        token: 34,
        name: "using",
        kind: DefaultKind::Keyword,
    },
    LangEntry {
        token: 35,
        name: "namespace",
        kind: DefaultKind::Keyword,
    },
    LangEntry {
        token: 36,
        name: "class",
        kind: DefaultKind::Keyword,
    },
    LangEntry {
        token: 37,
        name: "struct",
        kind: DefaultKind::Keyword,
    },
    LangEntry {
        token: 38,
        name: "interface",
        kind: DefaultKind::Keyword,
    },
    LangEntry {
        token: 39,
        name: "enum",
        kind: DefaultKind::Keyword,
    },
    LangEntry {
        token: 40,
        name: "return",
        kind: DefaultKind::Keyword,
    },
    LangEntry {
        token: 41,
        name: "if",
        kind: DefaultKind::Keyword,
    },
    LangEntry {
        token: 42,
        name: "else",
        kind: DefaultKind::Keyword,
    },
    LangEntry {
        token: 43,
        name: "for",
        kind: DefaultKind::Keyword,
    },
    LangEntry {
        token: 44,
        name: "foreach",
        kind: DefaultKind::Keyword,
    },
    LangEntry {
        token: 45,
        name: "while",
        kind: DefaultKind::Keyword,
    },
    LangEntry {
        token: 46,
        name: "do",
        kind: DefaultKind::Keyword,
    },
    LangEntry {
        token: 47,
        name: "switch",
        kind: DefaultKind::Keyword,
    },
    LangEntry {
        token: 48,
        name: "case",
        kind: DefaultKind::Keyword,
    },
    LangEntry {
        token: 49,
        name: "break",
        kind: DefaultKind::Keyword,
    },
    LangEntry {
        token: 50,
        name: "continue",
        kind: DefaultKind::Keyword,
    },
    LangEntry {
        token: 51,
        name: "goto",
        kind: DefaultKind::Keyword,
    },
    LangEntry {
        token: 52,
        name: "throw",
        kind: DefaultKind::Keyword,
    },
    LangEntry {
        token: 53,
        name: "try",
        kind: DefaultKind::Keyword,
    },
    LangEntry {
        token: 54,
        name: "catch",
        kind: DefaultKind::Keyword,
    },
    LangEntry {
        token: 55,
        name: "finally",
        kind: DefaultKind::Keyword,
    },
    LangEntry {
        token: 56,
        name: "lock",
        kind: DefaultKind::Keyword,
    },
    LangEntry {
        token: 57,
        name: "checked",
        kind: DefaultKind::Keyword,
    },
    LangEntry {
        token: 58,
        name: "unchecked",
        kind: DefaultKind::Keyword,
    },
    LangEntry {
        token: 59,
        name: "fixed",
        kind: DefaultKind::Keyword,
    },
    LangEntry {
        token: 60,
        name: "sizeof",
        kind: DefaultKind::Keyword,
    },
    LangEntry {
        token: 61,
        name: "typeof",
        kind: DefaultKind::Keyword,
    },
    LangEntry {
        token: 62,
        name: "is",
        kind: DefaultKind::Keyword,
    },
    LangEntry {
        token: 63,
        name: "as",
        kind: DefaultKind::Keyword,
    },
    LangEntry {
        token: 64,
        name: "var",
        kind: DefaultKind::Keyword,
    },
    LangEntry {
        token: 65,
        name: "get",
        kind: DefaultKind::Keyword,
    },
    LangEntry {
        token: 66,
        name: "set",
        kind: DefaultKind::Keyword,
    },
    LangEntry {
        token: 67,
        name: "add",
        kind: DefaultKind::Keyword,
    },
    LangEntry {
        token: 68,
        name: "remove",
        kind: DefaultKind::Keyword,
    },
    LangEntry {
        token: 69,
        name: "yield",
        kind: DefaultKind::Keyword,
    },
    LangEntry {
        token: 70,
        name: "await",
        kind: DefaultKind::Keyword,
    },
    LangEntry {
        token: 71,
        name: "when",
        kind: DefaultKind::Keyword,
    },
    LangEntry {
        token: 72,
        name: "where",
        kind: DefaultKind::Keyword,
    },
    LangEntry {
        token: 73,
        name: "global",
        kind: DefaultKind::Keyword,
    },
    LangEntry {
        token: 74,
        name: "record",
        kind: DefaultKind::Keyword,
    },
    LangEntry {
        token: 75,
        name: "init",
        kind: DefaultKind::Keyword,
    },
    LangEntry {
        token: 76,
        name: "required",
        kind: DefaultKind::Modifier,
    },
    LangEntry {
        token: 77,
        name: "?:",
        kind: DefaultKind::Construct,
    },
    LangEntry {
        token: 78,
        name: "??",
        kind: DefaultKind::Construct,
    },
    LangEntry {
        token: 79,
        name: "??=",
        kind: DefaultKind::Construct,
    },
    LangEntry {
        token: 80,
        name: "?.",
        kind: DefaultKind::Construct,
    },
    LangEntry {
        token: 81,
        name: "=>",
        kind: DefaultKind::Construct,
    },
    LangEntry {
        token: 82,
        name: "new()",
        kind: DefaultKind::Construct,
    },
    LangEntry {
        token: 83,
        name: "tuple",
        kind: DefaultKind::Construct,
    },
    LangEntry {
        token: 84,
        name: "discard",
        kind: DefaultKind::Construct,
    },
    LangEntry {
        token: 85,
        name: "range",
        kind: DefaultKind::Construct,
    },
    LangEntry {
        token: 86,
        name: "index-from-end",
        kind: DefaultKind::Construct,
    },
    LangEntry {
        token: 87,
        name: "using-declaration",
        kind: DefaultKind::Construct,
    },
    LangEntry {
        token: 88,
        name: "nameof",
        kind: DefaultKind::Construct,
    },
    LangEntry {
        token: 89,
        name: "switch-expression",
        kind: DefaultKind::Construct,
    },
    LangEntry {
        token: 90,
        name: "with-expression",
        kind: DefaultKind::Construct,
    },
    LangEntry {
        token: 91,
        name: "init-accessor",
        kind: DefaultKind::Construct,
    },
    LangEntry {
        token: 92,
        name: "throw-expression",
        kind: DefaultKind::Construct,
    },
    LangEntry {
        token: 93,
        name: "lambda",
        kind: DefaultKind::Construct,
    },
    LangEntry {
        token: 94,
        name: "interpolated-string",
        kind: DefaultKind::Construct,
    },
    LangEntry {
        token: 95,
        name: "verbatim-string",
        kind: DefaultKind::Construct,
    },
    LangEntry {
        token: 96,
        name: "raw-string",
        kind: DefaultKind::Construct,
    },
    LangEntry {
        token: 97,
        name: "deconstruction",
        kind: DefaultKind::Construct,
    },
    LangEntry {
        token: 98,
        name: "checked-expression",
        kind: DefaultKind::Construct,
    },
    LangEntry {
        token: 99,
        name: "pattern",
        kind: DefaultKind::Construct,
    },
];

const SYSTEM_TYPES: &[&str] = &[
    "System.Object",
    "System.String",
    "System.Int32",
    "System.UInt32",
    "System.Int64",
    "System.UInt64",
    "System.Int16",
    "System.UInt16",
    "System.Byte",
    "System.SByte",
    "System.Single",
    "System.Double",
    "System.Decimal",
    "System.Boolean",
    "System.Char",
    "System.Void",
    "System.Type",
    "System.Array",
    "System.Enum",
    "System.Delegate",
    "System.Exception",
    "System.Collections.Generic.List",
    "System.Collections.Generic.Dictionary",
    "System.Collections.Generic.HashSet",
    "System.Collections.Generic.IEnumerable",
    "System.Collections.Generic.IList",
    "System.Collections.Generic.IDictionary",
    "System.Linq.Enumerable",
    "System.Threading.Tasks.Task",
    "System.Nullable",
    "System.Guid",
    "System.DateTime",
    "System.TimeSpan",
    "UnityEngine.Object",
    "UnityEngine.GameObject",
    "UnityEngine.Transform",
    "UnityEngine.Component",
    "UnityEngine.MonoBehaviour",
    "UnityEngine.Vector2",
    "UnityEngine.Vector3",
    "UnityEngine.Vector4",
    "UnityEngine.Quaternion",
    "UnityEngine.Color",
    "UnityEngine.Mathf",
    "UnityEngine.Time",
    "UnityEngine.Debug",
    "UnityEngine.ScriptableObject",
    "UnityEngine.Animator",
    "VRC.SDKBase.VRCPlayerApi",
    "VRC.SDKBase.Networking",
    "System.ValueType",
    "System.MulticastDelegate",
    "System.Attribute",
    "System.IntPtr",
    "System.UIntPtr",
    "System.Random",
    "System.Math",
    "System.Convert",
    "System.Environment",
    "System.Console",
    "System.Buffer",
    "System.BitConverter",
    "System.Uri",
    "System.Version",
    "System.Lazy",
    "System.WeakReference",
    "System.GC",
    "System.AppDomain",
    "System.Activator",
    "System.Tuple",
    "System.ValueTuple",
    "System.Span",
    "System.Memory",
    "System.ReadOnlySpan",
    "System.ReadOnlyMemory",
    "System.ArraySegment",
    "System.Index",
    "System.Range",
    "System.HashCode",
    "System.StringComparer",
    "System.StringComparison",
    "System.StringSplitOptions",
    "System.Text.Encoding",
    "System.Text.StringBuilder",
    "System.Globalization.CultureInfo",
    "System.IFormatProvider",
    "System.IComparable",
    "System.IEquatable",
    "System.IFormattable",
    "System.ICloneable",
    "System.IConvertible",
    "System.IDisposable",
    "System.IAsyncDisposable",
    "System.IServiceProvider",
    "System.ArgumentException",
    "System.ArgumentNullException",
    "System.ArgumentOutOfRangeException",
    "System.InvalidOperationException",
    "System.NotSupportedException",
    "System.NotImplementedException",
    "System.NullReferenceException",
    "System.IndexOutOfRangeException",
    "System.KeyNotFoundException",
    "System.FormatException",
    "System.OverflowException",
    "System.DivideByZeroException",
    "System.ObjectDisposedException",
    "System.TimeoutException",
    "System.OperationCanceledException",
    "System.AggregateException",
    "System.Collections.IEnumerable",
    "System.Collections.IEnumerator",
    "System.Collections.ICollection",
    "System.Collections.IList",
    "System.Collections.IDictionary",
    "System.Collections.ArrayList",
    "System.Collections.Hashtable",
    "System.Collections.BitArray",
    "System.Collections.Generic.IEnumerator",
    "System.Collections.Generic.ICollection",
    "System.Collections.Generic.ISet",
    "System.Collections.Generic.IReadOnlyCollection",
    "System.Collections.Generic.IReadOnlyList",
    "System.Collections.Generic.IReadOnlyDictionary",
    "System.Collections.Generic.IComparer",
    "System.Collections.Generic.IEqualityComparer",
    "System.Collections.Generic.SortedSet",
    "System.Collections.Generic.Queue",
    "System.Collections.Generic.Stack",
    "System.Collections.Generic.LinkedList",
    "System.Collections.Generic.SortedDictionary",
    "System.Collections.Generic.SortedList",
    "System.Collections.Generic.KeyValuePair",
    "System.Collections.Concurrent.ConcurrentDictionary",
    "System.Collections.Concurrent.ConcurrentQueue",
    "System.Collections.Concurrent.ConcurrentStack",
    "System.Collections.Concurrent.ConcurrentBag",
    "System.Collections.ObjectModel.ObservableCollection",
    "System.Collections.ObjectModel.ReadOnlyCollection",
    "System.Linq.IQueryable",
    "System.Linq.IGrouping",
    "System.Linq.ILookup",
    "System.Linq.Lookup",
    "System.IO.Stream",
    "System.IO.FileStream",
    "System.IO.MemoryStream",
    "System.IO.TextReader",
    "System.IO.TextWriter",
    "System.IO.StreamReader",
    "System.IO.StreamWriter",
    "System.IO.BinaryReader",
    "System.IO.BinaryWriter",
    "System.IO.File",
    "System.IO.Directory",
    "System.IO.Path",
    "System.IO.FileInfo",
    "System.IO.DirectoryInfo",
    "System.IO.FileSystemWatcher",
    "System.IO.Compression.GZipStream",
    "System.IO.Compression.DeflateStream",
    "System.Text.RegularExpressions.Regex",
    "System.Text.RegularExpressions.Match",
    "System.Text.RegularExpressions.Group",
    "System.Text.RegularExpressions.Capture",
    "System.Text.RegularExpressions.MatchCollection",
    "System.Text.ASCIIEncoding",
    "System.Text.UTF8Encoding",
    "System.Text.UnicodeEncoding",
    "System.Net.WebClient",
    "System.Net.Http.HttpClient",
    "System.Net.WebRequest",
    "System.Net.WebResponse",
    "System.Net.IPAddress",
    "System.Net.Dns",
    "System.Net.Sockets.Socket",
    "System.Net.Sockets.TcpClient",
    "System.Net.Sockets.UdpClient",
    "System.Net.UriBuilder",
    "System.Reflection.Assembly",
    "System.Reflection.MethodInfo",
    "System.Reflection.PropertyInfo",
    "System.Reflection.FieldInfo",
    "System.Reflection.TypeInfo",
    "System.Reflection.MemberInfo",
    "System.Reflection.ParameterInfo",
    "System.Reflection.ConstructorInfo",
    "System.Reflection.EventInfo",
    "System.Reflection.Module",
    "System.Reflection.BindingFlags",
    "System.Text.Json.JsonSerializer",
    "System.Text.Json.JsonDocument",
    "System.Text.Json.JsonElement",
    "System.Text.Json.JsonNode",
    "System.Text.Json.JsonObject",
    "System.Text.Json.JsonArray",
    "System.Text.Json.JsonValue",
    "System.Text.Json.JsonSerializerOptions",
    "System.Text.Json.Serialization.JsonConverter",
    "System.Xml.XmlDocument",
    "System.Xml.Linq.XDocument",
    "System.Xml.Linq.XElement",
    "System.Diagnostics.Debug",
    "System.Diagnostics.Trace",
    "System.Diagnostics.Stopwatch",
    "System.Diagnostics.Process",
    "System.Diagnostics.ProcessStartInfo",
    "System.Diagnostics.Activity",
    "System.Diagnostics.ActivitySource",
    "System.Security.Cryptography.MD5",
    "System.Security.Cryptography.SHA1",
    "System.Security.Cryptography.SHA256",
    "System.Security.Cryptography.SHA512",
    "System.Security.Cryptography.RandomNumberGenerator",
    "System.Security.Cryptography.Aes",
    "System.Security.Cryptography.RSA",
    "System.Security.Cryptography.HMACSHA256",
    "System.Numerics.BigInteger",
    "System.Numerics.Complex",
    "System.Numerics.Vector2",
    "System.Numerics.Vector3",
    "System.Numerics.Vector4",
    "System.Threading.Thread",
    "System.Threading.Tasks.TaskCompletionSource",
    "System.Threading.Tasks.ValueTask",
    "System.Threading.CancellationToken",
    "System.Threading.CancellationTokenSource",
    "System.Threading.Timer",
    "System.Threading.Interlocked",
    "System.Threading.Monitor",
    "System.Threading.Mutex",
    "System.Threading.Semaphore",
    "System.Threading.SemaphoreSlim",
    "System.Threading.ReaderWriterLockSlim",
    "System.Threading.ThreadPool",
    "System.Threading.ManualResetEvent",
    "System.Threading.AutoResetEvent",
    "System.Threading.WaitHandle",
    "UnityEngine.RectTransform",
    "UnityEngine.Behaviour",
    "UnityEngine.Renderer",
    "UnityEngine.MeshRenderer",
    "UnityEngine.SkinnedMeshRenderer",
    "UnityEngine.MeshFilter",
    "UnityEngine.Mesh",
    "UnityEngine.Material",
    "UnityEngine.Shader",
    "UnityEngine.Texture",
    "UnityEngine.Texture2D",
    "UnityEngine.RenderTexture",
    "UnityEngine.Sprite",
    "UnityEngine.SpriteRenderer",
    "UnityEngine.Camera",
    "UnityEngine.Light",
    "UnityEngine.AudioSource",
    "UnityEngine.AudioClip",
    "UnityEngine.Rigidbody",
    "UnityEngine.Collider",
    "UnityEngine.BoxCollider",
    "UnityEngine.SphereCollider",
    "UnityEngine.CapsuleCollider",
    "UnityEngine.MeshCollider",
    "UnityEngine.CharacterController",
    "UnityEngine.Rigidbody2D",
    "UnityEngine.Collider2D",
    "UnityEngine.Canvas",
    "UnityEngine.CanvasGroup",
    "UnityEngine.Rect",
    "UnityEngine.Bounds",
    "UnityEngine.Ray",
    "UnityEngine.RaycastHit",
    "UnityEngine.Plane",
    "UnityEngine.Matrix4x4",
    "UnityEngine.Color32",
    "UnityEngine.Gradient",
    "UnityEngine.AnimationCurve",
    "UnityEngine.Application",
    "UnityEngine.Resources",
    "UnityEngine.SceneManagement.SceneManager",
    "UnityEngine.SceneManagement.Scene",
    "UnityEngine.AssetBundle",
    "UnityEngine.Animation",
    "UnityEngine.Coroutine",
    "UnityEngine.WaitForSeconds",
    "UnityEngine.WaitForEndOfFrame",
    "UnityEngine.YieldInstruction",
    "UnityEngine.CustomYieldInstruction",
    "UnityEngine.UI.Image",
    "UnityEngine.UI.Text",
    "UnityEngine.UI.Button",
    "UnityEngine.UI.Slider",
    "UnityEngine.UI.Toggle",
    "UnityEngine.UI.InputField",
    "UnityEngine.UI.Dropdown",
    "UnityEngine.UI.ScrollRect",
    "UnityEngine.UI.LayoutGroup",
    "UnityEngine.EventSystems.EventSystem",
    "UnityEngine.EventSystems.PointerEventData",
    "UnityEditor.Editor",
    "UnityEditor.EditorWindow",
    "UnityEditor.SerializedObject",
    "UnityEditor.SerializedProperty",
    "UnityEditor.AssetDatabase",
    "UnityEditor.Selection",
    "UnityEditor.Undo",
    "UnityEditor.PrefabUtility",
    "UnityEditor.EditorGUILayout",
    "UnityEditor.EditorGUI",
    "UnityEngine.GUILayout",
    "UnityEngine.GUI",
    "UnityEngine.GUIStyle",
    "UnityEngine.GUISkin",
    "UnityEngine.GUILayoutOption",
    "VRC.SDKBase.VRCPlayerApi.TrackingData",
    "VRC.SDKBase.VRCPlayerApi.TrackingDataType",
    "VRC.SDKBase.VRCStation",
    "VRC.SDKBase.VRC_AvatarDescriptor",
    "VRC.SDKBase.VRC_SpatialAudioSource",
    "VRC.SDKBase.VRC_MirrorReflection",
    "VRC.SDKBase.VRC_Pickup",
    "VRC.SDKBase.VRC_SceneDescriptor",
    "VRC.SDK3.Components.VRCStation",
    "VRC.SDK3.Components.VRCPhysBone",
    "VRC.SDK3.Components.VRCPhysBoneCollider",
    "VRC.SDK3.Components.VRCContactSender",
    "VRC.SDK3.Components.VRCContactReceiver",
    "VRC.SDK3.Avatars.Components.VRCAvatarDescriptor",
    "VRC.SDK3.Data.DataDictionary",
    "VRC.SDK3.Data.DataList",
    "VRC.SDK3.Data.DataToken",
    "VRC.SDK3.StringLoading",
    "VRC.SDK3.ImageLoading",
    "VRC.SDK3.Video",
    "VRC.Udon.UdonBehaviour",
    "VRC.Udon.Common.Interfaces",
    "UdonSharp.UdonSharpBehaviour",
];

const DEFAULT_METHODS: &[&str] = &[
    "System.Console.WriteLine",
    "System.Console.ReadLine",
    "System.Math.Abs",
    "System.Math.Max",
    "System.Math.Min",
    "System.String.Format",
    "System.String.Join",
    "System.String.IsNullOrEmpty",
    "System.String.IsNullOrWhiteSpace",
    "System.String.Concat",
    "System.Linq.Enumerable.ToList",
    "System.Linq.Enumerable.ToArray",
    "System.Linq.Enumerable.Where",
    "System.Linq.Enumerable.Select",
    "System.Linq.Enumerable.Any",
    "System.Linq.Enumerable.All",
    "System.Linq.Enumerable.Count",
    "System.Collections.Generic.List.Add",
    "System.Collections.Generic.List.Remove",
    "System.Collections.Generic.List.Contains",
    "System.Collections.Generic.List.ToArray",
    "System.Collections.Generic.Dictionary.TryGetValue",
    "System.Collections.Generic.Dictionary.Add",
    "System.Object.ToString",
    "System.Object.Equals",
    "System.Object.GetHashCode",
    "System.IDisposable.Dispose",
    "UnityEngine.Debug.Log",
    "UnityEngine.Debug.LogWarning",
    "UnityEngine.Debug.LogError",
    "UnityEngine.Vector3.zero",
    "UnityEngine.Quaternion.identity",
    "UnityEngine.Time.deltaTime",
    "UnityEngine.Mathf.Clamp",
    "UnityEngine.Mathf.Lerp",
    "UnityEngine.Component.GetComponent",
    "UnityEngine.GameObject.SetActive",
    "UnityEngine.Object.Instantiate",
    "UnityEngine.Object.Destroy",
    "VRC.SDKBase.VRCPlayerApi.GetPlayerById",
    "VRC.SDKBase.VRCPlayerApi.isLocal",
    "VRC.SDKBase.Networking.SetOwner",
    "VRC.SDKBase.Networking.LocalPlayer",
    "System.Collections.Generic.List.Count",
    "System.Collections.Generic.Dictionary.Count",
    "System.String.Length",
    "System.Array.Length",
    "System.Math.Clamp",
    "System.Math.Round",
    "System.Convert.ToInt32",
    "System.String.Substring",
    "System.String.Replace",
    "System.String.Split",
    "System.String.Trim",
    "System.String.ToLower",
    "System.String.ToUpper",
    "System.String.StartsWith",
    "System.String.EndsWith",
    "System.String.Contains",
    "System.String.IndexOf",
    "System.Collections.Generic.List.Insert",
    "System.Collections.Generic.List.RemoveAt",
    "System.Collections.Generic.List.Clear",
    "System.Collections.Generic.Dictionary.Remove",
    "System.Collections.Generic.Dictionary.ContainsKey",
    "System.Linq.Enumerable.First",
    "System.Linq.Enumerable.FirstOrDefault",
    "System.Linq.Enumerable.Last",
    "System.Linq.Enumerable.OrderBy",
    "System.Linq.Enumerable.ToDictionary",
    "System.Math.Pow",
    "System.Math.Sqrt",
    "System.Math.Floor",
    "System.Math.Ceiling",
    "System.Random.Next",
    "System.Convert.ToSingle",
    "System.Convert.ToString",
    "UnityEngine.Vector3.Distance",
    "UnityEngine.Vector3.Normalize",
    "UnityEngine.Quaternion.Euler",
    "UnityEngine.Quaternion.LookRotation",
    "UnityEngine.Transform.SetParent",
    "UnityEngine.GameObject.Find",
    "UnityEngine.Mathf.Round",
    "UnityEngine.Mathf.Floor",
    "UnityEngine.Mathf.Ceil",
    "UnityEngine.Mathf.Abs",
    "UnityEngine.Mathf.Sign",
];

static TABLE: OnceLock<Vec<DefaultToken>> = OnceLock::new();

fn build_table() -> Vec<DefaultToken> {
    assert!(
        SYSTEM_TYPES.len() as u32 <= SYSTEM_TYPE_CAPACITY,
        "SYSTEM_TYPES overflows its {} reserved slots",
        SYSTEM_TYPE_CAPACITY
    );
    assert!(
        DEFAULT_METHODS.len() as u32 <= DEFAULT_METHOD_CAPACITY,
        "DEFAULT_METHODS overflows its {} reserved slots",
        DEFAULT_METHOD_CAPACITY
    );
    let mut v = Vec::with_capacity(LANGUAGE.len() + SYSTEM_TYPES.len() + DEFAULT_METHODS.len());
    for e in LANGUAGE {
        v.push(DefaultToken {
            token: e.token,
            name: e.name,
            kind: e.kind,
        });
    }
    for (i, name) in SYSTEM_TYPES.iter().enumerate() {
        v.push(DefaultToken {
            token: SYSTEM_TYPE_BASE + i as u32,
            name,
            kind: DefaultKind::Type,
        });
    }
    for (i, name) in DEFAULT_METHODS.iter().enumerate() {
        v.push(DefaultToken {
            token: DEFAULT_METHOD_BASE + i as u32,
            name,
            kind: DefaultKind::Method,
        });
    }
    v
}

/// Every statically defined token, in token order.
pub fn all() -> &'static [DefaultToken] {
    TABLE.get_or_init(build_table)
}

/// Resolve a token number to its static entry.
pub fn get(token: u32) -> Option<&'static DefaultToken> {
    all().iter().find(|e| e.token == token)
}

/// Resolve an exact full/name to its static token number.
///
/// C# aliases (`int`, `string`, `bool`, …) resolve to the same token as their
/// `System.*` type; they are the same type, so they must not get a second
/// number.
pub fn token(name: &str) -> Option<u32> {
    let name = canonical(name);
    all().iter().find(|e| e.name == name).map(|e| e.token)
}

/// Resolve an exact full/name to its static entry.
pub fn entry(name: &str) -> Option<&'static DefaultToken> {
    let name = canonical(name);
    all().iter().find(|e| e.name == name)
}

/// Reduce a declared type to the name that can be looked up.
///
/// `System.Collections.Generic.List<NZK.B>` → `System.Collections.Generic.List`,
/// `NZK.B[]` → `NZK.B`, `System.Nullable<int>` → `System.Nullable`,
/// `global::System.String` → `System.String`, `ref int` → `int`.
///
/// This is a *lookup* normalisation: the token still names the generic type or
/// element type, never a fabricated instantiation token.
pub fn type_base(name: &str) -> &str {
    let mut s = name.trim();
    if let Some(rest) = s.strip_prefix("global::") {
        s = rest;
    }
    for prefix in ["ref ", "in ", "out ", "params "] {
        if let Some(rest) = s.strip_prefix(prefix) {
            s = rest.trim_start();
            break;
        }
    }
    if let Some(i) = s.find('<') {
        s = &s[..i];
    }
    while let Some(rest) = s.strip_suffix("[]") {
        s = rest;
    }
    if let Some(rest) = s.strip_suffix('?') {
        s = rest;
    }
    s.trim()
}

/// Map a C# type alias to the canonical `System.*` spelling used by the table.
///
/// This is deliberately a *name* normalisation, not extra tokens: `g(102)` is
/// `System.Int32` whether the source wrote `int` or `System.Int32`.
pub fn canonical(name: &str) -> &str {
    match name {
        "bool" => "System.Boolean",
        "byte" => "System.Byte",
        "sbyte" => "System.SByte",
        "char" => "System.Char",
        "decimal" => "System.Decimal",
        "double" => "System.Double",
        "float" => "System.Single",
        "int" => "System.Int32",
        "uint" => "System.UInt32",
        "long" => "System.Int64",
        "ulong" => "System.UInt64",
        "short" => "System.Int16",
        "ushort" => "System.UInt16",
        "object" => "System.Object",
        "string" => "System.String",
        "dynamic" => "System.Object",
        "nint" => "System.Int32",
        "nuint" => "System.UInt32",
        _ => name,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn tokens_and_names_are_unique() {
        let mut tokens = HashSet::new();
        let mut names = HashSet::new();
        for e in all() {
            assert!(
                tokens.insert(e.token),
                "duplicate token {} ({})",
                e.token,
                e.name
            );
            assert!(names.insert(e.name), "duplicate name {}", e.name);
        }
    }

    #[test]
    fn table_is_in_token_order_and_inside_its_ranges() {
        let mut last = 0;
        for e in all() {
            assert!(e.token > last, "{} out of order", e.name);
            assert!(e.token <= DEFAULT_MAX, "{} above DEFAULT_MAX", e.name);
            last = e.token;
        }
        for e in all() {
            match e.kind {
                DefaultKind::Type => {
                    assert!(
                        (SYSTEM_TYPE_FIRST..=SYSTEM_TYPE_LAST).contains(&e.token),
                        "type {} outside the system-type range",
                        e.name
                    );
                }
                DefaultKind::Method => {
                    assert!(
                        (DEFAULT_METHOD_FIRST..=DEFAULT_MAX).contains(&e.token),
                        "method {} outside the default-method range",
                        e.name
                    );
                }
                DefaultKind::Construct => {
                    assert!(
                        (77..=99).contains(&e.token),
                        "construct {} outside 77..=99",
                        e.name
                    );
                }
                _ => {}
            }
        }
    }

    #[test]
    fn owner_fixed_numbers_hold() {
        assert_eq!(token("public"), Some(PUBLIC));
        assert_eq!(token("private"), Some(PRIVATE));
        assert_eq!(token("static"), Some(STATIC));
        assert_eq!(token("const"), Some(CONST));
        assert_eq!(token("void"), Some(VOID));
        assert_eq!(token("return"), Some(RETURN));
        assert_eq!(token("System.Object"), Some(100));
        assert_eq!(token("System.String"), Some(101));
        assert_eq!(token("?:"), Some(77));
        assert_eq!(token("??"), Some(78));
        assert_eq!(token("=>"), Some(81));
    }

    #[test]
    fn type_base_strips_generics_arrays_nullable_and_ref() {
        assert_eq!(
            type_base("System.Collections.Generic.List<NZK.B>"),
            "System.Collections.Generic.List"
        );
        assert_eq!(type_base("NZK.B[]"), "NZK.B");
        assert_eq!(type_base("System.Nullable<int>"), "System.Nullable");
        assert_eq!(type_base("global::System.String"), "System.String");
        assert_eq!(type_base("ref int"), "int");
        assert_eq!(type_base(" int? "), "int");
    }

    #[test]
    fn csharp_aliases_share_the_system_type_token() {
        for (alias, full) in [
            ("int", "System.Int32"),
            ("string", "System.String"),
            ("bool", "System.Boolean"),
            ("float", "System.Single"),
            ("object", "System.Object"),
        ] {
            assert_eq!(token(alias), token(full), "{alias} vs {full}");
        }
        assert_eq!(token("int"), Some(102));
        assert_eq!(token("string"), Some(101));
    }

    #[test]
    fn system_types_are_all_before_default_methods_and_project_tokens() {
        let last_type = all()
            .iter()
            .filter(|e| e.kind == DefaultKind::Type)
            .map(|e| e.token)
            .max()
            .unwrap();
        let first_method = all()
            .iter()
            .filter(|e| e.kind == DefaultKind::Method)
            .map(|e| e.token)
            .min()
            .unwrap();
        assert!(last_type < first_method);
        assert!(last_type < crate::dict::FIRST_PROJECT_TOKEN);
        assert!(first_method < crate::dict::FIRST_PROJECT_TOKEN);
    }

    #[test]
    fn the_system_type_list_is_broad() {
        // A regression guard for "pre-assign all system types": the table must
        // stay well above the original 50-entry stub.
        assert!(
            SYSTEM_TYPES.len() >= 250,
            "only {} system types",
            SYSTEM_TYPES.len()
        );
    }
}
