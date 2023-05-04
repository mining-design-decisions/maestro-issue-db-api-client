import typing


class IssueAPIError(Exception):
    ...

class InvalidCredentialsException(IssueAPIError):
    ...

class NotAuthorizedException(IssueAPIError):
    ...

class InvalidTokenException(IssueAPIError):
    ...

class HTTPException(IssueAPIError):
    ...

class LibraryException(IssueAPIError):
    ...


class IssueRepository:
    def __init__(self,
                 url: str, *,
                 credentials: tuple[str, str] | None = None,
                 label_caching_policy: str = 'no_caching',
                 config_handling_policy: str = 'read_fetch_write_fetch',
                 allow_self_signed_certificates: bool = False):
        ...

    def __repr__(self) -> str:
        ...

    def search(self, /,
               q: Query, *,
               attributes: list[str] = (),
               load_labels: bool = False) -> list[Issue]:
        ...

    @property
    def repos(self) -> list[Repo]:
        ...

    @property
    def tags(self) -> list[Tag]:
        ...

    def add_new_tag(self, name: str, description: str):
        ...

    @property
    def embeddings(self) -> list[Embedding]:
        ...

    def get_embedding_by_id(self, id: str) -> Embedding:
        ...

    def create_embedding(self, name: str, config: dict[str, typing.Any]) -> Embedding:
        ...

    def delete_embedding(self, embedding: Embedding):
        ...

    def find_issues_by_key(self, *args: tuple[str, str]) -> list[Issue]:
        ...

    @property
    def models(self) -> list[Model]:
        ...

    def get_model_by_id(self, id: str) -> Model:
        ...

    def add_model(self, name: str, config: dict[str, typing.Any]) -> Model:
        ...

    def delete_model_config(self, model: Model):
        ...


class Repo:
    def __repr__(self) -> str:
        ...

    @property
    def name(self) -> str:
        ...

    @property
    def projects(self) -> str:
        ...


class Query:
    def __init__(self):
        ...

    def __repr__(self) -> str:
        ...

    def lor(self, *args: Query) -> Query:
        ...

    def land(self, *args: Query) -> Query:
        ...

    def tag(self, name: str) -> Query:
        ...

    def not_tag(self, name: str) -> Query:
        ...


class Label:

    def __init__(self,
                 existence: bool,
                 executive: bool,
                 property: bool):
        ...

    def __repr__(self) -> str:
        ...

    def __eq__(self, other) -> bool | NotImplemented:
        ...

    def __hash__(self) -> int:
        ...

    @property
    def existence(self) -> bool:
        ...

    @property
    def executive(self) -> bool:
        ...

    @property
    def non_architectural(self) -> bool:
        ...


    @property
    def property(self) -> bool:
        ...


class Issue:
    def __new__(cls):
        ...

    def __repr__(self) -> str:
        ...

    def __eq__(self, other) -> bool | NotImplemented:
        ...

    def __hash__(self) -> int:
        ...

    @property
    def identifier(self) -> str:
        ...

    @property
    def manual_label(self) -> Label:
        ...

    @manual_label.setter
    def manual_label(self, value: Label):
        ...

    def invalidate_label_cache(self):
        ...

    @property
    def tags(self) -> list[str]:
        ...

    def add_tag(self, name: str):
        ...

    def remove_tag(self, name: str):
        ...

    @property
    def is_in_review(self) -> bool:
        ...

    @is_in_review.setter
    def is_in_review(self, value: bool):
        ...

    @property
    def labelling_comments(self) -> list[Comment]:
        ...

    def add_labelling_comment(self, text: str):
        ...

    def remove_labelling_comment(self, comment: Comment):
        ...

    @property
    def key(self) -> str:
        ...

    @property
    def summary(self) -> str:
        ...

    @property
    def description(self) -> str:
        ...

    @property
    def comments(self) -> list[str]:
        ...

    @property
    def status(self) -> str:
        ...

    @property
    def priority(self) -> str:
        ...

    @property
    def resolution(self) -> str | None:
        ...

    @property
    def issue_type(self) -> str:
        ...

    @property
    def issue_links(self) -> list[Issue]:
        ...

    @property
    def parent(self) -> Issue | None:
        ...

    @property
    def subtasks(self) -> list[Issue]:
        ...

    @property
    def watches(self) -> int:
        ...

    @property
    def votes(self) -> int:
        ...

    @property
    def date_created(self) -> str:
        ...

    @property
    def date_updated(self) -> str:
        ...

    @property
    def date_resolved(self) -> str:
        ...

    @property
    def labels(self) -> list[str]:
        ...

    @property
    def components(self) -> list[str]:
        ...

    @property
    def fix_versions(self) -> list[str]:
        ...

    @property
    def affected_versions(self) -> list[str]:
        ...


class Comment:
    def __repr__(self) -> str:
        ...

    @property
    def author(self) -> str:
        ...

    @property
    def body(self) -> str:
        ...

    @body.setter
    def body(self, text: str):
        ...


class Tag:
    def __repr__(self) -> str:
        ...

    def __eq__(self, other) -> bool | NotImplemented:
        ...

    def __hash__(self) -> int:
        ...

    @property
    def name(self) -> str:
        ...

    @property
    def description(self) -> str:
        ...

    @description.setter
    def description(self, description: str):
        ...

    @property
    def tag_type(self) -> str:
        ...


class Embedding:
    def __repr__(self) -> str:
        ...

    def __eq__(self, other) -> bool | NotImplemented:
        ...

    @property
    def identifier(self) -> str:
        ...

    @property
    def name(self) -> str:
        ...

    @name.setter
    def name(self, name: str):
        ...

    @property
    def config(self) -> dict[str, typing.Any]:
        ...

    @config.setter
    def config(self, value: dict[str, typing.Any]):
        ...

    def download_binary(self, path: str):
        ...

    def upload_binary(self, path: str):
        ...

    def delete_binary(self):
        ...


class Model:

    def __repr__(self) -> str:
        ...

    def __eq__(self, other) -> bool:
        ...

    def __hash__(self) -> int:
        ...

    @property
    def identifier(self) -> str:
        ...

    @property
    def name(self) -> str:
        ...

    @name.setter
    def name(self, name: str):
        ...

    @property
    def config(self) -> dict[str, typing.Any]:
        ...

    @config.setter
    def config(self, config: dict[str, typing.Any]):
        ...

    @property
    def versions(self) -> list[Version]:
        ...

    def get_version_by_id(self, id: str) -> Version:
        ...

    def add_version(self, path: str, description: str | None = None) -> Version:
        ...

    def remove_version(self, version: Version):
        ...

    @property
    def test_runs(self) -> list[TestRun]:
        ...

    def get_run_by_id(self, id: str) -> TestRun:
        ...

    def add_test_run(self,
                     data: list[typing.Any],
                     description: str | None = None) -> TestRun:
        ...

    def delete_test_run(self, run: TestRun):
        ...


class Version:

    def __repr__(self) -> str:
        ...

    def __eq__(self, other) -> bool:
        ...

    @property
    def model_id(self) -> str:
        ...

    @property
    def version_id(self) -> str:
        ...

    def download(self, path: str):
        ...

    @property
    def predictions(self) -> dict[str, typing.Any]:
        ...

    @predictions.setter
    def predictions(self, predictions: dict[str, typing.Any]):
        ...

    def delete_predictions(self):
        ...

    @property
    def description(self) -> str:
        ...

    @description.setter
    def description(self, description: str):
        ...


class TestRun:

    def __repr__(self) -> str:
        ...

    @property
    def model_id(self) -> str:
        ...

    @property
    def run_id(self) -> str:
        ...

    @property
    def data(self) -> list[typing.Any]:
        ...

    @property
    def description(self) -> str:
        ...

    @description.setter
    def description(self, description: str):
        ...
