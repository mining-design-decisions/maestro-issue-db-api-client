from setuptools import setup
from setuptools_rust import Binding, RustExtension

setup(
    rust_extensions=[
        RustExtension("issue_db_api.issue_api", path='issue_db_api/Cargo.toml', binding=Binding.PyO3, debug=False)
    ],
    zip_safe=False
)
