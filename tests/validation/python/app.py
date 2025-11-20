import requests
import yaml
import click
# NOTE: pillow is NOT imported - should be unreachable

@click.command()
def main():
    # Using requests - REACHABLE vulnerability
    response = requests.get('https://api.github.com')
    click.echo(f'Status: {response.status_code}')

    # Using pyyaml - REACHABLE vulnerability
    data = yaml.safe_load('key: value')
    click.echo(f'Parsed: {data}')

    # NOTE: pillow is never used
    # The vulnerability should be marked as UNREACHABLE

if __name__ == '__main__':
    main()
