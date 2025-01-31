import os
import typer

from pathlib import Path

from text_generation import server, utils

app = typer.Typer()


@app.command()
def serve(
    model_name: str,
    sharded: bool = False,
    quantize: bool = False,
    uds_path: Path = "/tmp/text-generation",
):
    if sharded:
        assert (
            os.getenv("RANK", None) is not None
        ), "RANK must be set when sharded is True"
        assert (
            os.getenv("WORLD_SIZE", None) is not None
        ), "WORLD_SIZE must be set when sharded is True"
        assert (
            os.getenv("MASTER_ADDR", None) is not None
        ), "MASTER_ADDR must be set when sharded is True"
        assert (
            os.getenv("MASTER_PORT", None) is not None
        ), "MASTER_PORT must be set when sharded is True"

    server.serve(model_name, sharded, quantize, uds_path)


@app.command()
def download_weights(
    model_name: str,
    extension: str = ".safetensors",
):
    utils.download_weights(model_name, extension)


if __name__ == "__main__":
    app()
